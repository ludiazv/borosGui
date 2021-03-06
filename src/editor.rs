use serde::Deserialize;
use iui::prelude::*;
use iui::controls::{Control, Spinbox,Entry,Combobox,Checkbox,
                    VerticalBox,HorizontalBox,Window,Label,
                    HorizontalSeparator,TabGroup, Button, Spacer };

use crate::ser::BorosSerial;
use crate::Actions;
use crate::devices::yml;
use serde_yaml::{Result,from_str};
use regex::Regex;

use std::sync::mpsc::Sender;




#[derive(Deserialize)]
pub struct Root {
    spec: Vec<Device>,
}

#[derive(Deserialize)]
pub struct Device {
    signature: Signature,
    title: String,
    sections: Vec<Section>,
}

impl Device {
    pub fn find(&mut self, id: &str) -> Option<&mut ConfItem> {
        for s in &mut self.sections {
            let f=s.find(id);
            if f.is_some() { return f }
        }
        None
    }
}

#[derive(Deserialize,Debug)]
pub struct Signature {
    product: String,
    model: String,
    version: i32,
}
impl Signature {
    pub fn new(p:&str,m:&str,v:&str) -> Self {
        let v = v.parse::<i32>().unwrap_or(1i32);
        Self {
            product: String::from(p),
            model: String::from(m),
            version: v,
        }
    }
}
impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.product == other.product && self.model==other.model && self.version==self.version
    }
}


#[derive(Deserialize)]
pub struct Section {
    name: String,
    help: String,
    items: Vec<ConfItem>
}

impl Section {
    pub fn find(&mut self, s:&str) -> Option<&mut ConfItem> {
        let f=self.items.iter_mut().find( |e| e.is(s));
        f
    }
}

#[derive(Deserialize)]
pub struct Choice {
    pub val: i32,
    pub desc: String,
}

#[derive(Deserialize)]
pub enum ConfItem {
    Int    { id: String, caption:String, val:i32,  vmax:i32, vmin:i32 ,
            #[serde(skip)]
             control: Option<Spinbox>,
           },
    Hex    { id: String, caption:String,val:String, maxlen:usize, lsb:bool,
            #[serde(skip)]
            control: Option<Entry>,
           },
    Text   { id: String, caption:String, val:String, maxlen:usize,
            #[serde(skip)]
            control: Option<Entry>,
           },
    Choice { id: String, caption:String ,val:usize,  values:Vec<Choice> ,
            #[serde(skip)]
            control: Option<Combobox>,
           },
    Check  { id: String, caption:String, val:bool,
             #[serde(skip)]
             control: Option<Checkbox>,
           }
}

impl ConfItem {
    pub fn is(&self,wid:&str) -> bool {
        match self {
            ConfItem::Check { id , .. } |
            ConfItem::Int { id , .. } |
            ConfItem::Hex { id, .. } |
            ConfItem::Choice { id, .. } |
            ConfItem::Text { id , .. } => id==wid
        }
    }

    pub fn validate(&self,ui:&UI) -> (bool,&String) {
        match self {
            ConfItem::Check { caption, .. } => (true,caption),
            ConfItem::Int { caption, .. } => (true,caption),
            ConfItem::Choice { control:Some(c), caption , .. } => (c.selected(ui) >=0,caption) ,
            ConfItem::Text { control:Some(c), maxlen: ml , caption, .. } => (c.value(ui).len() <= *ml , caption) ,
            ConfItem::Hex { control: Some(c), maxlen: ml, caption, ..} => {
                let v= c.value(ui);
                //println!("v:{} , len:{}",v,v.len());
                ( (v.len() <= (2*ml) && ConfItem::is_hex(&v)) , caption)
            },
            ConfItem::Text { control:None , caption, .. } |
            ConfItem::Hex { control:None , caption, .. } |
            ConfItem::Choice { control:None , caption, .. }  => (false,caption)

        }
    }
    
    fn  is_hex(s:&str) -> bool {
        let re = Regex::new(r"[0123456789abcdefABCDEF]").unwrap();
        //println!("slen {} count {}", s.len() , re.find_iter(s).count() );
        s.len() >0 && s.len() % 2 == 0 && re.find_iter(s).count() == s.len()
    }
    fn invert(s:&str) -> String {
        let re = Regex::new(r"[0123456789abcdefABCDEF][0123456789abcdefABCDEF]").unwrap();
        let mut x :Vec<&str>=re.find_iter(s).map(|x| { x.as_str() }).collect();
        x.reverse();
        x.join("")
    }

    pub fn from_device(&mut self,ui:&UI,v:&str) {
        match self {
            ConfItem::Check { control: Some(c), .. } => c.set_checked(ui,v == "1" ) ,
            ConfItem::Int   { control: Some(c), val:def, .. } => {
                let v=v.parse::<i32>().unwrap_or(*def);
                c.set_value(ui,v);
            },
            ConfItem::Choice { control: Some(c), values:options, val:def, .. } => {
                let vp=v.parse::<i32>().unwrap_or(0);
                let idx=options.iter().position( |o| o.val==vp ).unwrap_or(*def);
                c.set_selected(ui,idx as i32); 
            },
            ConfItem::Text { control: Some(c), .. } => c.set_value(ui,v),
            ConfItem::Hex  { control: Some(c), lsb: l, ..} => {
                if *l { 
                    c.set_value(ui,ConfItem::invert(v).as_str()) 
                } else { 
                    c.set_value(ui,v)
                }
            }
            _ => { }
        } // Match
    }

    pub fn to_device(&self,ui:&UI) -> String {
        match self {
            ConfItem::Check { control:Some(c), id, .. } => { 
                let v= if c.checked(ui) { "1" } else { "0" };
                format!("{} {}",id,v)
            },
            ConfItem::Int { control:Some(c), id,  .. } => format!("{} {}",id,c.value(ui)),
            ConfItem::Hex { control:Some(c), lsb: l, id,  .. } => {
                let mut v=c.value(ui);
                if *l { v= ConfItem::invert(&v) }
                format!("{} {}",id,v)
            },
            ConfItem::Text   { control:Some(c), id,  .. } => format!("{} {}",id,c.value(ui)),
            ConfItem::Choice { control:Some(c), values:v,  id, .. } => {
                let idx = c.selected(ui);
                format!("{} {}",id,v[idx as usize].val)
            },
            _ => "".into()  
        }
    }

    pub fn build_control(&mut self,ui:&UI) -> HorizontalBox {
        let mut hb=HorizontalBox::new(ui);
        let (caption,control) : (&str,Control) = match self {
            ConfItem::Text { control: c , val: v, caption: cap, ..} => {
                 let mut con = Entry::new(ui);
                 con.set_value(ui,v);
                 let rcon=con.clone();
                 *c=Some(con);
                 (cap, rcon.into())
            },
            ConfItem::Int { control: c , vmax:vma, vmin: vmi ,caption:cap, val:v, ..} => {
                let mut con= Spinbox::new(ui,*vmi,*vma);
                con.set_value(ui,*v);
                let rcon=con.clone();
                *c=Some(con);
                (cap,rcon.into())
            },
            ConfItem::Check { control: c , val: v, caption: cap, ..} => {
                let mut con = Checkbox::new(ui,cap);
                con.set_checked(ui,*v);
                let rcon=con.clone();
                *c=Some(con);
                ("",rcon.into())
           },
           ConfItem::Hex { control: c , val: v, caption: cap, ..} => {
                let mut con = Entry::new(ui);
                con.set_value(ui,v);
                let rcon=con.clone();
                *c=Some(con);
                (cap, rcon.into())
           },
           ConfItem::Choice {control:c, val: idx, caption:cap, values: vals,.. } => {
               let mut con = Combobox::new(ui);
               for v in vals {
                   con.append(ui,v.desc.as_str());
               }
               con.set_selected(ui,*idx as i32);
               let rcon=con.clone();
               *c=Some(con);
               (cap,rcon.into())
           },
            //_ => ( "",HorizontalSeparator::new(ui).into() )
        };

        if caption != ""  {
            hb.append(ui,Label::new(ui,caption),LayoutStrategy::Compact);
        }
        hb.append(ui,control,LayoutStrategy::Stretchy);
        hb.set_padded(ui,true);
        hb
    }

}    

pub struct Editor {
    root: Root,
    ui: UI,
    win: Window,
    serial: Option<Box<BorosSerial>>,
    info: Label,
    cmd: Sender<Actions>,
    aspec: usize,

}

impl Editor {
    pub fn new(ui : UI,cmd :Sender<Actions>) -> Result<Self> {
        #[cfg(debug_assertions)] let str=std::fs::read_to_string("./spec.yml").unwrap();
        #[cfg(not(debug_assertions))] let str=String::from(yml);
        let r= from_str(str.as_str())?;
        let win= Window::new(&ui, "Config editor", 640, 380, WindowType::NoMenubar);
        let info=Label::new(&ui,"Ready");
        Ok(Self {
            root: r,
            ui: ui,
            win: win,
            serial: None,
            info: info,
            cmd: cmd,
            aspec: 0,
        })
    }
    pub fn take_serial(&mut self,ser:Box<BorosSerial>) {
        self.serial=Some(ser);
    }
    pub fn get_and_check_signature(&mut self) -> usize {
        if let Some(ser) = &mut self.serial {
            if ser.connect() {
                if let Ok(sig)=ser.get_signature() {
                    return self.check_signature(&sig);
                }
            } 
        }
        usize::MAX
    }
    pub fn check_signature(&self,sig: &Signature ) -> usize {
        for (i,e) in self.root.spec.iter().enumerate() {
            if e.signature == *sig {
                return i;
            }
        }
        usize::MAX
    }

    pub fn reset(&mut self) {
        if let Some(ser) = &mut self.serial {
            let (ok,_) = ser.do_cmd("fac").unwrap_or((false,vec!()));
            if ok {
                self.editor_info("Factory settings done");
            } else {
                self.editor_info("Factory settings failed.")
            }
        }
    }

    pub fn editor_info(&mut self,s: &str) {
        self.info.set_text(&self.ui,s);
    }
    pub fn read_config(&mut self) {
        if let Some(ser) = &mut self.serial {
            
            if let Ok(config) = ser.get_config() {
                for (id,val) in config {
                    if let Some(item)=self.root.spec[self.aspec].find(id.as_str()) {
                        item.from_device(&self.ui, val.as_str())
                    }
                }
                self.editor_info("Config readed from device!");
            } else {
                self.editor_info("Failed to read configuration from device");
            }
        }
    }

    pub fn save_config(&mut self) {
        if let Some(ser) = &mut self.serial {
            for sec in &self.root.spec[self.aspec].sections {
                for item in sec.items.iter() {
                    let (valid,caption) = item.validate(&self.ui);
                    if valid {
                        let ser_cmd=item.to_device(&self.ui);
                        if ser.do_cmd(&ser_cmd).is_err() {
                            let msg=format!("The field '{}' in tab '{}' can't be written into the device",caption,sec.name);
                            self.win.modal_err(&self.ui,"Field invalid",msg.as_str());
                            self.editor_info("¡¡¡ Error writing cofiguration");
                            return;
                        }
                    } else {
                        let msg=format!("The field '{}' in tab '{}' is not valid. Check format and length.",caption,sec.name);
                        self.win.modal_err(&self.ui,"Field invalid",msg.as_str());
                        self.editor_info("¡¡¡ Invalid fields");
                        return;
                    }
                }
            }
            self.editor_info("Config written to device!");
        }

    }

    pub fn show(&mut self,n: usize) {
        self.aspec=n;
        let model= &mut self.root.spec[n];
        let ui=&self.ui;
        self.win.set_title(ui,model.title.as_str());
        let mut tabs= TabGroup::new(ui);
        
        for sec in &mut model.sections {
            let mut tab=VerticalBox::new(ui);
            tab.set_padded(ui,true);
            for i in &mut sec.items {
                tab.append(ui,i.build_control(ui),LayoutStrategy::Stretchy);
            }
            let mut help = Button::new(ui,"Help");
            tab.append(ui,HorizontalSeparator::new(ui),LayoutStrategy::Compact);
            
            help.on_clicked(ui, {
                let ui=ui.clone();
                let w=self.win.clone();
                let h=sec.help.clone();
                move |_| {
                    w.modal_msg(&ui,"Help",h.as_str());
                }
            });

            tab.append(ui,help,LayoutStrategy::Compact);
            let n=tabs.append(ui,&sec.name,tab);
            tabs.set_margined(ui,n-1,true);
        }

        let mut vbox=VerticalBox::new(ui);
        vbox.set_padded(ui,true);
        vbox.append(ui,self.info.clone(),LayoutStrategy::Compact);
        vbox.append(ui,tabs,LayoutStrategy::Compact);
        let mut bbox=HorizontalBox::new(ui);
        bbox.set_padded(ui,true);
        let mut quit=Button::new(ui,"Quit");
        let mut reset=Button::new(ui,"Factory reset");
        let mut read=Button::new(ui,"Read configuration");
        let mut write=Button::new(ui,"Write configuration");
        
        quit.on_clicked(&ui, {
            let ui = ui.clone();
            move |_| {
                ui.quit();
            }
        });

        reset.on_clicked(ui, {
            let c = self.cmd.clone();
            move |_| {
                let _=c.send(Actions::EditorInfo("Factory reset...".into()));
                let _=c.send(Actions::Reset);
                let _=c.send(Actions::EditorInfo("Reading config...".into()));
                let _=c.send(Actions::ReadConfig);
            }
        });
        read.on_clicked(ui, {
            let c=self.cmd.clone();
            move |_| {
                let _=c.send(Actions::EditorInfo("Reading config...".into()));
                let _=c.send(Actions::ReadConfig);
            }
        });
        write.on_clicked(ui, {
          let c=self.cmd.clone();
          move |_| {
              let _=c.send(Actions::EditorInfo("Writing configuration..".into()));
              let _=c.send(Actions::SaveConfig);
          }
        });

        bbox.append(ui,quit,LayoutStrategy::Compact);
        bbox.append(ui,reset,LayoutStrategy::Compact);
        bbox.append(ui,read,LayoutStrategy::Compact);
        bbox.append(ui,write,LayoutStrategy::Compact);
        
        vbox.append(ui,Spacer::new(ui),LayoutStrategy::Stretchy);
        vbox.append(ui,bbox,LayoutStrategy::Compact);

        self.win.set_child(ui,vbox);
        self.win.show(ui);
        let _=self.cmd.send(Actions::EditorInfo("Reading config...".into()));
        let _ =self.cmd.send(Actions::ReadConfig);
    }

}
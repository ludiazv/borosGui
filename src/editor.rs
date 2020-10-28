use serde::Deserialize;
use iui::prelude::*;
use iui::controls::{Control, Spinbox,Entry,Combobox,Checkbox,
                    VerticalBox,HorizontalBox,Window,Label,
                    HorizontalSeparator,TabGroup, Button, Spacer };

use crate::ser::BorosSerial;
use crate::Actions;
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


#[derive(Deserialize)]
pub struct Choice {
    val: i32,
    desc: String,
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
    pub fn validate(&self,ui:&UI) -> bool {
        match self {
            ConfItem::Check { .. } => true,
            ConfItem::Int {  .. } => true, 
            ConfItem::Choice { control:Some(c), .. } => c.selected(ui) >=0 ,
            ConfItem::Text { control:Some(c), maxlen: ml , .. } => c.value(ui).len() <= *ml ,
            ConfItem::Hex { control: Some(c), maxlen: ml, ..} => {
                false // TODO
            },
            _ => false   
        }
    }

    pub fn from_device(&mut self,ui:&UI,v:&str) {
        match self {
            ConfItem::Check { control: Some(c), .. } => c.set_checked(ui,v == "1" ) ,
            ConfItem::Int   { control: Some(c), val:def, .. } => {
                let v=v.parse::<i32>().unwrap_or(*def);
                c.set_value(ui,v);
            },
            ConfItem::Choice { control: Some(c), values:options, val:def, .. } => {
                let idx=v.parse::<usize>().unwrap_or(*def);
                if idx < options.len() { c.set_selected(ui,idx as i32); }
            },
            ConfItem::Text { control: Some(c), .. } => c.set_value(ui,v),
            ConfItem::Hex  { control: Some(c), lsb: l, ..} => {
                if ! *l {
                    c.set_value(ui,v);
                }
            }
            _ => { }
        } // Match
    }

    pub fn to_device(&self,ui:&UI) -> String {
        String::from("todo")
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
            _ => { ("",HorizontalSeparator::new(ui).into())}
        };
        if caption != ""  {
            hb.append(ui,Label::new(ui,caption),LayoutStrategy::Stretchy);
        }
        hb.append(ui,control,LayoutStrategy::Stretchy);
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
        let re=Regex::new(r"[0123456789abcdef][0123456789abcdef]").unwrap();
        let mut x :Vec<&str>=re.find_iter("12abcc").map(|x| { x.as_str() }).collect();
        x.reverse();
        println!("{:?}",x.join(""));
        let str=std::fs::read_to_string("./spec.yml").unwrap();
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
            let (ok,lines) = ser.do_cmd("show").unwrap_or((false,vec!()));
            if ok {
                for l in &lines {
                    
                }
            }
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

        bbox.append(ui,quit,LayoutStrategy::Compact);
        bbox.append(ui,reset,LayoutStrategy::Compact);
        bbox.append(ui,read,LayoutStrategy::Compact);
        bbox.append(ui,write,LayoutStrategy::Compact);
        
        vbox.append(ui,Spacer::new(ui),LayoutStrategy::Stretchy);
        vbox.append(ui,bbox,LayoutStrategy::Compact);

        self.win.set_child(ui,vbox);
        self.win.show(ui);

    }

}
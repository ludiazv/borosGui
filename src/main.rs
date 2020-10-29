use serialport::{available_ports,open,SerialPortType,UsbPortInfo};
use std::sync::mpsc::channel;

extern crate iui;
use iui::prelude::*;
use iui::controls::{Label, Button, VerticalBox, HorizontalBox,Combobox,ProgressBar};

use ser::BorosSerial;
use editor::Editor;

mod ser;
mod editor;
mod devices;

pub enum Actions {
    OpenEditor(String),
    PBShow,
    PBHide,
    EditorInfo(String),
    Reset,
    ReadConfig,
    SaveConfig,
}


fn main() {
    

    let ui : UI = UI::init().unwrap();
    let (cmd_sender,cmd_receiver) = channel::<Actions>();
    let mut editor : Editor = Editor::new(ui.clone(),cmd_sender.clone()).unwrap();
    
    let mut w_select = Window::new(&ui, "Choose serial", 320, 200, WindowType::NoMenubar);

     // Layout & group for select window
     let mut vbox = VerticalBox::new(&ui);
     vbox.set_padded(&ui, true);
     let mut device_combo= Combobox::new(&ui);
     let ports=available_ports().unwrap();
     
     for i in ports.iter() {
         device_combo.append(&ui,&i.port_name);
     }
    
    let info= Label::new(&ui,"Select serial interface");
    let mut progress = ProgressBar::indeterminate(&ui);
    progress.hide(&ui);

    device_combo.on_selected(&ui, {
        let ui=ui.clone();
        let mut inf=info.clone();
        let p=ports.clone();
        move |i| {
            if i>=0 {
                let s:String=match &p[i as usize].port_type {
                    SerialPortType::BluetoothPort => "Bluetooth interface".into(),
                    SerialPortType::PciPort => "Pci interface".into(),
                    SerialPortType::Unknown => "Unknown interface ".into(),
                    SerialPortType::UsbPort(UsbPortInfo { manufacturer, product , ..}) => {
                        format!("USB Vendor:{},Product:{}",manufacturer.as_ref().unwrap_or(&"#".to_string()),product.as_ref().unwrap_or(&"#".to_string()))
                    }
                };
                inf.set_text(&ui,&s);
            }
        }
    });
    if ports.len() > 0 {
        device_combo.set_selected(&ui,1); // Todo change to 0
    }

    let mut group_hbox = HorizontalBox::new(&ui);
    let mut but_go = Button::new(&ui,"Go!");
    but_go.on_clicked(&ui, {
        let ui = ui.clone();
        let dc=device_combo.clone();
        let w=w_select.clone();

        move |_| {
            let n=dc.selected(&ui);
            if n == -1 {
                w.modal_err(&ui,"Error","No serial interface selected");
            } else {
                let _= cmd_sender.send(Actions::PBShow);
                let _= cmd_sender.send(Actions::OpenEditor(ports[n as usize].port_name.clone()));
            }
        }
    });

    let mut quit_button = Button::new(&ui, "Quit");
    quit_button.on_clicked(&ui, {
        let ui = ui.clone();
        move |_| {
            ui.quit();
        }
    });

    vbox.append(&ui, device_combo, LayoutStrategy::Stretchy);
    vbox.append(&ui, info, LayoutStrategy::Stretchy);
    vbox.append(&ui, progress.clone(), LayoutStrategy::Stretchy);
    group_hbox.append(&ui, quit_button, LayoutStrategy::Stretchy);
    group_hbox.append(&ui, but_go, LayoutStrategy::Stretchy);
    //group.set_child(&ui, group_hbox);
    vbox.append(&ui, group_hbox, LayoutStrategy::Stretchy);
    //vbox.append(&ui, bar, LayoutStrategy::Stretchy);

    // Show the window
    w_select.set_child(&ui, vbox);
    w_select.show(&ui);
    // Run the application
    let mut event_loop = ui.event_loop();
    event_loop.on_tick(&ui, {
        let mut w=w_select.clone();
        let ui=ui.clone();
        let mut pb=progress.clone();
        move || {
            if let Ok(msg)=cmd_receiver.try_recv() {
                match msg {
                    Actions::OpenEditor(dev) => {
                        if let Ok(ser)=open(&dev) {
                            editor.take_serial(Box::new(BorosSerial::new(ser)));
                            let n =editor.get_and_check_signature();
                            if n< usize::MAX {
                                editor.show(n);
                                w.hide(&ui);
                            } else {
                                w.modal_err(&ui,"Error","Couldn't not retrieve a valid signature of the device");
                            }
                        } else {
                            w.modal_err(&ui,"Error","Couldn't open serial interface");
                        }
                        pb.hide(&ui);
                    },
                    Actions::PBHide => pb.hide(&ui),
                    Actions::PBShow => pb.show(&ui),
                    Actions::EditorInfo(s) => editor.editor_info(&s),
                    Actions::Reset  => editor.reset(),
                    Actions::ReadConfig => editor.read_config(),
                    Actions::SaveConfig => editor.save_config(),
                    //_ => {}
                }
            }
        }
    });
    event_loop.run(&ui);

}

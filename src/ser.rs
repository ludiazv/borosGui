use serialport::{SerialPort};
use std::time::Duration;
use std::thread::sleep;
use std::io::{Result,Error,ErrorKind};
use regex::Regex;

use crate::editor::Signature;

pub struct BorosSerial {
    port : Box<dyn SerialPort>,
    prompt: [u8;1],
    found_prompt: bool,
}

impl BorosSerial {

    pub fn new(serial: Box<dyn SerialPort>) -> Self {
        Self {
            port: serial,
            prompt: [b'>'],
            found_prompt: false,
        }
    }
    pub fn connect(&mut self) -> bool {
        let _=self.port.set_timeout(Duration::from_secs(2));
        // Reset via DTR 
        sleep(Duration::from_millis(500));
        let _=self.port.write_data_terminal_ready(true);
        sleep(Duration::from_millis(100));
        let _=self.port.write_data_terminal_ready(false);
        sleep(Duration::from_millis(1500));
        // wait for prompt
        self.wait_prompt()
    }

    fn wait_prompt(&mut self) -> bool {
        let mut c: [u8;1] = [0;1];
        self.found_prompt = false;
        while self.port.read_exact(&mut c).is_ok()  {
            //println!("{}",c[0]);
            if c[0] == b'\n' && self.port.read_exact(&mut c).is_ok() && c == self.prompt {
                self.found_prompt=true;
                break;
            }
        }
        self.found_prompt
    }

    pub fn do_cmd(&mut self,cmd:&str) -> Result<(bool,Vec<String>)> {
        if !self.found_prompt {
            Err(Error::from(ErrorKind::NotConnected))
        } else {
            let prompt=std::str::from_utf8(&self.prompt).unwrap();
            sleep(Duration::from_millis(100));
            self.found_prompt=false;
            self.port.write(cmd.as_bytes())?;
            self.port.write(&[b'\n'])?;
            sleep(Duration::from_millis(500));
            let mut buf = String::new();
            let mut c: [u8;1] = [0;1];
            while self.port.read_exact(&mut c).is_ok() {
                buf.push(c[0] as char)
            }
            let mut lines : Vec<String>=buf.split('\n').filter_map(|x| {
               let tr=x.trim();
               if tr.is_empty() || tr== cmd {
                    None
               } else {
                   Some(tr.to_string())
               }
            }).collect();
            // Detect promt & remove it from ouput
            self.found_prompt=lines.iter().find( |x| *x==prompt).is_some();
            lines.retain( |x| *x!=prompt);
            let res=lines.iter().find( |x| *x=="[OK]" );

            Ok((res.is_some(),lines))
        }
    }

    pub fn get_signature(&mut self) -> Result<Signature> {
        let (res,lines) = self.do_cmd("ver")?;
        if res && lines.len()>=1 {
            let rx= Regex::new(r"(.+)\[(.+)<(.+)>V(\d+)\](.+)").unwrap();
            //println!("{:?} , {:?}",lines[0], rx.captures(lines[0].as_str()));
            if let Some(cap)= rx.captures(lines[0].as_str()) {
                return Ok(Signature::new(&cap[2],&cap[3],&cap[4]));
            }
        } 
        Err(Error::new(ErrorKind::Other, "Can't read device signature"))
    
    }



}

/*
    
                

        

    def get_config(self,cmd='show'):
        ok,attrs= self.do_cmd(cmd)
        if not ok: raise Exception("can't retrieve configuration")
        cnf={}
        for a in attrs:
            m= re.match(r'^\[(.+)\].*:(.+)',a)
            if m is not None: cnf[m.group(1)]=m.group(2)
        return cnf
        */

#![recursion_limit = "1024"]

#[macro_use] extern crate error_chain;
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! { 
    
        foreign_links {
            Libusb(::libusb::Error);
        }
    }
}
use errors::*;


#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate structopt;
#[macro_use] extern crate structopt_derive;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use structopt::StructOpt;


#[derive(Deserialize, Debug)]
struct Config {
    keyboards: HashMap<String, Keyboard>,
    usbids: HashMap<String, String>,
}


#[derive(Deserialize, Debug)]
struct Keyboard {
    mapping: String,
    variant: String,
    options: Vec<String>,
    xmodmapconfig: String,
}


impl Keyboard {
    fn xkbmap_command_args(&self) -> String {
        return format!("{} -variant {}", self.mapping, self.variant)
    }

    fn setxkbmap_args(&self) -> Vec<String> {
        let mut command: Vec<String> = Vec::new();
        command.push(self.xkbmap_command_args());
        for opt in self.options.iter() {
            command.push(format!("{}", opt));
        }
        return command;
    }

    fn xmodmap_file(&self, path: &str) -> String {
        let mut command = String::new();
        command.push_str(path);
        command.push_str(self.xmodmapconfig.as_str());
        return command;
    }
}


#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(short = "d", long = "default", default_value = "104", help="Set the default keyboard.")]
    default: String,

    #[structopt(long = "debug", help="Don't execute, just show the commands.")]
    debug: bool,

    #[structopt(short = "l", long = "list", help="Show all keyboard configurations.")]
    list: bool,

    #[structopt(short = "a", long = "attached", help="Show all attached keyboards.")]
    attached: bool,
}

    
extern crate libusb;
use std::process::Command;
use std::str;


fn get_usb_ids() -> Result<Vec<String>> {
    let usbctx = libusb::Context::new()?;
    let mut result: Vec<String> = Vec::new();
    for device in usbctx.devices()?.iter() {
        let dev_desc = device.device_descriptor()?;
        result.push(format!("{:04x}:{:04x}", dev_desc.vendor_id(), dev_desc.product_id()));
    }
    Ok(result)
}


fn set_keyboard(keyboard: &Keyboard, debug: bool) {
    let xmodmap_path = "/home/timm.behner/Nextcloud/xmodmapconfigs/";
    if debug {
        println!("Command: setxkbmap {:?}", keyboard.setxkbmap_args());
        println!("xmodmap file: {}", keyboard.xmodmap_file(xmodmap_path));
    } else {
        Command::new("setxkbmap")
            .args(keyboard.setxkbmap_args())
            .output()
            .expect("Failed to execute process!");
        Command::new("xmodmap")
            .arg(keyboard.xmodmap_file(xmodmap_path))
            .output()
            .expect("Failed to execute process!");
    }
}


fn open(path: &str) -> Result<File> {
    File::open(path).chain_err(|| format!("Can't open `{}`", path))
}


fn read_file(path: &str) -> Result<String> {
    let mut result = String::new();
    let mut file = open(path)?;
    file.read_to_string(&mut result).chain_err(|| format!("Can't read `{}`", path));
    Ok(result)
}


quick_main!(run);


fn run() -> Result<()> {
    let cli = Cli::from_args();
    // TODO this should not be hard coded, but read from ENV
    let config_content = read_file("/home/timm.behner/dotfiles/keyset.toml")?;
    let config: Config = toml::from_str(config_content.as_str()).chain_err(|| "Unable to parse toml config")?;

    if cli.list {
        println!("All configured keyboards:");
        for (k, _) in config.keyboards.iter() {
            println!("{}", k);
        }
        return Ok(())
    }

    let usbids = get_usb_ids()?;
    if cli.attached {
        for usb_id in usbids.iter() {
            match config.usbids.get(usb_id){
                Some(kb) => println!("{} => {}", usb_id, kb),
                None => {} ,
            }
        }
        return Ok(())
    }

    for id in usbids.iter() {
        match config.usbids.get(id) {
            Some(kb_name) => {
                match config.keyboards.get(kb_name) {
                    Some(kb) => set_keyboard(kb, cli.debug),
                    None => return Err(format!("{} is missing a configuration", kb_name).into()),
                }
            },
            None => {}
        }
    }

    Ok(())
}



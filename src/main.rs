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

use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use structopt::StructOpt;
use std::path;


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
    fn setxkbmap_args(&self) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        args.push(self.mapping.clone());
        args.push(String::from("-variant"));
        args.push(self.variant.clone());
        for opt in self.options.iter() {
            args.push(String::from("-option"));
            args.push(opt.clone());
        }
        return args;
    }

    fn xmodmap_file(&self) -> Result<String> {
        let xmodmap_path = get_xmodmap_path()?;
        let xmodmap_config_path = xmodmap_path.join(&self.xmodmapconfig);

        if !xmodmap_config_path.is_file() {
            return Err("Xmodmap config does not exist!".into());
        }

        match xmodmap_config_path.to_str() {
            Some(p) => Ok(String::from(p)),
            None => Err("UTF-8 Error on xmodmap path".into())
        }
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

    #[structopt(short = "f", long = "force", help="Force a certain keyboard configuration.")]
    force: Option<String>
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


fn set_keyboard(keyboard: &Keyboard, debug: bool) -> Result<()> {
    let xmodmap_file = keyboard.xmodmap_file()?;
    if debug {
        println!("Command: setxkbmap {:?}", keyboard.setxkbmap_args());
        println!("xmodmap file: {}", xmodmap_file);
    } else {
        let out = Command::new("setxkbmap")
            .args(keyboard.setxkbmap_args())
            .output()
            .expect("Failed to execute process!");
        println!("status: {}", out.status);
        println!("stdout: {}", String::from_utf8_lossy(&out.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&out.stderr));
        // Command::new("xmodmap")
            // .arg(xmodmap_file)
            // .output()
            // .expect("Failed to execute process!");
    }
    Ok(())
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

fn get_config_path() -> Result<String> {
    let home_dir = match env::var_os("HOME") {
        Some(val) => val,
        None      => return Err("Dude! Why is HOME not set?!".into())
    };

    let mut config_path = path::PathBuf::new();
    config_path.push(home_dir);
    config_path.push("dotfiles/keyset.toml");

    if !config_path.is_file() {
        return Err("Config file does not exist!".into());
    }

    match config_path.to_str() {
        Some(path) => Ok(String::from(path)),
        None       => Err("Config path resulted in an UTF-8 error!".into()),
    }
}

fn get_xmodmap_path() -> Result<path::PathBuf> {
    let cloud_path = match env::var_os("CLOUDPATH"){
        Some(val) => val,
        None      => return Err("CLOUDPATH is not set".into())
    };

    let mut xmodmap_path = path::PathBuf::new();
    xmodmap_path.push(cloud_path);
    xmodmap_path.push("xmodmapconfigs");
    if !xmodmap_path.is_dir() {
        return Err("Xmodmappath is not a directory!".into());
    }
    Ok(xmodmap_path)
}


quick_main!(run);


fn run() -> Result<()> {
    let cli = Cli::from_args();
    // FIXME construct paths here, I mean real paths, from the aboves
    // need config_path and xmodmap_path

    let config_path = get_config_path()?;
    let config_content = read_file(config_path.as_str())?;
    let config: Config = toml::from_str(config_content.as_str()).chain_err(|| "Unable to parse toml config")?;

    if cli.list {
        println!("All configured keyboards:");
        for (k, _) in config.keyboards.iter() {
            println!("{}", k);
        }
        return Ok(())
    }

    match cli.force {
        Some(kb_name) => {
            println!("Forcing config {}", kb_name);
            match config.keyboards.get(&kb_name) {
                Some(kb) => return set_keyboard(kb, cli.debug),
                None     => return Err(format!("{} is not in your configuration!", kb_name).into()),
            };
        },
        None => {}
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
                    Some(kb) => return set_keyboard(kb, cli.debug),
                    None => return Err(format!("{} is missing a configuration", kb_name).into()),
                }
            },
            None => {}
        };
    }

    Ok(())
}



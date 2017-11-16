#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate toml;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Deserialize)]
struct Config {
    keyboards: HashMap<String, Keyboard>,
    usbids: HashMap<String, String>,
}

#[derive(Deserialize)]
struct Keyboard {
    mapping: String,
    variant: String,
    options: Vec<String>,
    xmodmapconfig: String,
}

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! { }
}
use errors::*;



extern crate libusb;
// use std::thread;
// use std::sync::mpsc;
// use std::process::Command;
use std::str;

// fn produce_ids(tx: mpsc::Sender<String>) {
//     let usbctx = libusb::Context::new().unwrap();
//     for device in usbctx.devices().unwrap().iter() {
//         let dev_desc = device.device_descriptor().unwrap();
//         tx.send(format!("{:04x}:{:04x}", dev_desc.vendor_id(), dev_desc.product_id()));
//     }
// }

// fn set_keyboard(keyboard_identifier: &str) {
//     let output = Command::new("echo")
//         .arg("Hello World")
//         .output()
//         .expect("Failed to execute process!");
//     println!("{}", str::from_utf8(&output.stdout).unwrap());
// }

quick_main!(run);

fn open(path: &str) -> Result<File> {
    File::open(path).chain_err(|| format!("Can't open `{}`", path))
}

fn read_file(path: &str) -> Result<String> {
    let mut result = String::new();
    let mut file = open(path)?;
    file.read_to_string(&mut result).chain_err(|| format!("Can't read `{}`", path));
    Ok(result)
}

fn run() -> Result<()> {
    // TODO this should not be hard coded, but read from ENV
    let config_content = read_file("/home/timm.behner/dotfiles/keyset.toml")?;
    let config: Config = toml::from_str(config_content.as_str()).chain_err(|| "Unable to parse toml config")?;
    for (k, v) in config.keyboards.iter() {
        println!("{} : {:?}", k, v.mapping);
    }

    for (id, conf) in config.usbids.iter() {
        println!("{} should be configured as {}", id, conf);
    }
    Ok(())
}



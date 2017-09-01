extern crate libusb;
use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;
use std::process::Command;
use std::str;

#[derive(Debug)]
struct KeyboardConfig {
    name: String,
    layout: String,
    variant: String,
}

fn produce_ids(tx: mpsc::Sender<String>) {
    let usbctx = libusb::Context::new().unwrap();
    for device in usbctx.devices().unwrap().iter() {
        let dev_desc = device.device_descriptor().unwrap();
        tx.send(format!("{:04x}:{:04x}", dev_desc.vendor_id(), dev_desc.product_id()));
    }
}

fn set_keyboard(keyboard_identifier: &str) {
    let output = Command::new("echo")
        .arg("Hello World")
        .output()
        .expect("Failed to execute process!");
    println!("{}", str::from_utf8(&output.stdout).unwrap());
}

fn main() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {produce_ids(tx)});

    let mut keyboard_configs: HashMap<String, KeyboardConfig> = HashMap::new();
    let mut keyboard_ids: HashMap<String, String> = HashMap::new();

    keyboard_configs.insert("Matias Ergo Pro".to_string(), 
                            KeyboardConfig{
                                name    : "Matias Ergo Pro".to_string(),
                                layout  : "us".to_string(),
                                variant : "alt-intl".to_string(),
                            }
                            );

    keyboard_ids.insert("05e3:0614".to_string(), "Matias Ergo Pro".to_string());

    for id in rx {
        match keyboard_ids.get(&id) {
            Some(name) => {
                println!("Found {}", name);
                println!("Using {:?}", keyboard_configs[name]);
                set_keyboard(name);
            },
            None => continue,
        }
    }
}

extern crate libusb;
// use std::thread;
// use std::sync::mpsc;
use std::collections::HashMap;
// use std::process::Command;
use std::str;

#[derive(Debug)]
struct KeyboardConfig {
    name: String,
    layout: String,
    variant: String,
}

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

fn main() {
}

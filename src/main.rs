extern crate libusb;
use std::thread;
use std::sync::mpsc;

fn produce_ids(tx: mpsc::Sender<String>) {
    let usbctx = libusb::Context::new().unwrap();
    for device in usbctx.devices().unwrap().iter() {
        let dev_desc = device.device_descriptor().unwrap();
        tx.send(format!("{:04x}:{:04x}", dev_desc.vendor_id(), dev_desc.product_id()));
    }
}

fn main() {
    let matias_ergo_pro_id = String::from("05e3:0614");
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {produce_ids(tx)});

    for id in rx {
        if id == matias_ergo_pro_id {
            println!("Found matias ergo pro: {}", id);
        }
    }
}

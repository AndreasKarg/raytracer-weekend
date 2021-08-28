use std::{io::Read, time::Duration};

use postcard::{take_from_bytes, take_from_bytes_cobs};
use raytracer_weekend_lib::Pixel;

fn main() {
    println!("Hello, world!");

    let port = serialport::new("COM3", 9600)
        .timeout(Duration::from_millis(10000))
        .open()
        .expect("Failed to open port");

    let mut buf = Vec::new();
    let mut bytes = port.bytes().peekable();

    // while *bytes.peek().unwrap().as_ref().unwrap() != 0x00 {
    //     println!("Skipping..!");
    //     bytes.next();
    // }

    for byte in bytes {
        let byte = byte.unwrap();
        buf.push(byte);

        //println!("Pushing byte {}, now at length {}", byte, buf.len());

        match take_from_bytes::<Pixel>(&mut buf) {
            Ok((pixel, residual)) => {
                println!("Got a pixel: {:#?}", pixel);
                buf = residual.to_vec();
                break;
            }
            Err(postcard::Error::DeserializeUnexpectedEnd) => {
                //println!("Not enough data...");
                continue;
            }
            Err(postcard::Error::DeserializeBadEncoding) => {
                //println!("WTF");
                continue;
            }
            e => {
                e.unwrap();
            }
        }
    }
}

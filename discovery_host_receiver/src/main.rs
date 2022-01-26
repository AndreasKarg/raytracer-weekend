#![feature(let_else)]

use std::{
    io::{Read, Write},
    time::Duration,
};

use postcard::{from_bytes_cobs, take_from_bytes, take_from_bytes_cobs};
use raytracer_weekend_lib::Pixel;
use serialport::ClearBuffer;

fn main() {
    println!("Hello, world!");

    let port = serialport::new("COM12", 9600)
        .timeout(Duration::from_millis(100000))
        .open()
        .expect("Failed to open port");

    port.clear(ClearBuffer::All);

    let mut bytes = port.bytes();

    loop {
        println!("Awaiting chunk...");
        let chunk: Result<Vec<_>, _> = bytes
            .by_ref()
            .map_while(|b| match b {
                Ok(0) => None,
                Ok(b) => Some(Ok(b)),
                err => Some(err),
            })
            .collect();
        let mut chunk = chunk.expect("Serial port error! WTF!");

        match from_bytes_cobs::<Pixel>(&mut chunk) {
            Ok(pixel) => {
                println!("Got a pixel: {:#?}", pixel);
            }
            Err(postcard::Error::DeserializeUnexpectedEnd) => {
                println!("Not enough data...");
                continue;
            }
            Err(postcard::Error::DeserializeBadEncoding) => {
                println!("WTF");
                continue;
            }
            e => {
                e.unwrap();
            }
        }
    }
}

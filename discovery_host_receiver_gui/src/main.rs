#![feature(let_else)]

use std::{cell::Cell, io::Read, time::Duration};

use dioxus::{
    core::exports::futures_channel::mpsc::{unbounded, UnboundedReceiver},
    prelude::*,
};
use image::Rgb;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use postcard::from_bytes_cobs;
use raytracer_weekend_lib::{Pixel, ProgressMessage};
use tokio_serial::{ClearBuffer, SerialPort};

fn main() {
    let (sender, receiver) = unbounded();

    let serial_port = tokio_serial::new("COM12", 115_200)
        .timeout(Duration::from_millis(1000000))
        .open()
        .expect("Failed to open port");

    serial_port.clear(ClearBuffer::All).unwrap();

    // launch our IO thread
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                serial_rx_loop(serial_port);
            });
    });

    // launch our app on the current thread - important because we spawn a window
    dioxus::desktop::launch_with_props(
        app,
        AppProps {
            receiver: Cell::new(Some(receiver)),
        },
        |c| c,
    )
}

struct AppProps {
    receiver: Cell<Option<UnboundedReceiver<Vec<()>>>>,
}

fn app(cx: Scope<AppProps>) -> Element {
    rsx!(cx, div { "Current stopwatch time: nom" })
}

fn serial_rx_loop(serial_port: Box<impl SerialPort + ?Sized>) -> ! {
    println!("Hello, world!");

    let mut bytes = serial_port.bytes();

    let mut state = None;

    loop {
        // println!("Awaiting chunk...");
        let chunk: Result<Vec<_>, _> = bytes
            .by_ref()
            .map_while(|b| match b {
                Ok(0) => None,
                Ok(b) => Some(Ok(b)),
                err => Some(err),
            })
            .collect();
        let mut chunk = chunk.expect("Serial port error! WTF!");

        let message = match from_bytes_cobs::<ProgressMessage>(&mut chunk) {
            Ok(message) => {
                // println!("Got a message: {:#?}", message);
                message
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
                unreachable!()
            }
        };

        match message {
            ProgressMessage::ImageStart {
                width,
                height,
                samples_per_pixel,
            } => {
                let progress_bar = ProgressBar::new((width * height) as u64);
                progress_bar.set_style(ProgressStyle::default_bar().template(
                    "[{elapsed_precise} / {eta_precise}/ {duration_precise}] {wide_bar:cyan/blue} {pos:>7}/{len:7} {msg}",
                ));
                progress_bar.set_position(0);

                state = Some((
                    image::DynamicImage::new_rgb8(width, height),
                    progress_bar,
                    samples_per_pixel,
                ));
            }
            ProgressMessage::Pixel(Pixel { row, column, color }) => {
                let Some((img, progress_bar, samples_per_pixel)) = state.as_mut() else {
                    continue;
                };

                let r = color.x();
                let g = color.y();
                let b = color.z();

                // Divide the color by the number of samples and gamma-correct for gamma=2.0.
                let scale = 1.0 / *samples_per_pixel as f32;
                let r = (scale * r).sqrt();
                let g = (scale * g).sqrt();
                let b = (scale * b).sqrt();

                let ir = (255.999 * r.clamp(0.0, 0.999)) as u8;
                let ig = (255.999 * g.clamp(0.0, 0.999)) as u8;
                let ib = (255.999 * b.clamp(0.0, 0.999)) as u8;

                let p = img.as_mut_rgb8().unwrap().get_pixel_mut(column, row);
                *p = Rgb([ir, ig, ib]);

                progress_bar.inc(1);
            }
            ProgressMessage::ImageEnd => {
                let Some((img, progress_bar, ..)) = state.as_mut() else {
                    continue;
                };

                progress_bar.finish();
                let rotated = img.rotate180();
                rotated.save("foo.png").unwrap();
            }
        }
    }
}

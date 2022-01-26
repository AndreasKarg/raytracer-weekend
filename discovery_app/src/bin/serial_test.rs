#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

use discovery_app as _; // global logger + panicking-behavior + memory layout

extern crate cortex_m;
#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
#[macro_use(block)]
extern crate nb;

extern crate stm32l4xx_hal as hal;
// #[macro_use(block)]
// extern crate nb;

use core::alloc::Layout;

use alloc_cortex_m::CortexMHeap;
use cortex_m::asm;
use postcard::to_slice_cobs;
// use postcard::to_slice_cobs;
use raytracer_weekend_lib::{vec3::Color, Pixel};

use crate::hal::{
    prelude::*,
    serial::{Config, Serial},
};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Hello, world!");

    let start = cortex_m_rt::heap_start() as usize;
    let size = 65536; // in bytes
    unsafe { ALLOCATOR.init(start, size) }

    let p = hal::stm32::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);

    let mut gpiod = p.GPIOD.split(&mut rcc.ahb2);
    // let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);

    // clock configuration using the default settings (all clocks run at 8 MHz)
    // let clocks = rcc.cfgr.freeze(&mut flash.acr, &mut pwr);
    // TRY this alternate clock configuration (clocks run at nearly the maximum frequency)
    let clocks = rcc
        .cfgr
        .sysclk(80.mhz())
        .pclk1(80.mhz())
        .pclk2(80.mhz())
        .freeze(&mut flash.acr, &mut pwr);

    // The Serial API is highly generic
    // TRY the commented out, different pin configurations
    // let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let tx = gpiod.pd5.into_af7(&mut gpiod.moder, &mut gpiod.afrl);
    // let tx = gpiob.pb6.into_af7(&mut gpiob.moder, &mut gpiob.afrl);

    // let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpiod.pd6.into_af7(&mut gpiod.moder, &mut gpiod.afrl);
    // let rx = gpiob.pb7.into_af7(&mut gpiob.moder, &mut gpiob.afrl);

    // TRY using a different USART peripheral here
    let serial = Serial::usart2(
        p.USART2,
        (tx, rx),
        Config::default().baudrate(9_600.bps()),
        clocks,
        &mut rcc.apb1r1,
    );
    let (mut tx, mut rx) = serial.split();

    let pixel = Pixel {
        row: 123,
        column: 456,
        color: Color::new(1.0, 2.0, 100.0),
    };
    // let pixel = "Foobar";

    let mut buf = [0u8; 256];

    let serialised = to_slice_cobs(&pixel, &mut buf).unwrap();

    // The `block!` macro makes an operation block until it finishes
    // NOTE the error type is `!`

    // block!(tx.write(sent)).ok();

    // let received = block!(rx.read()).unwrap();

    // assert_eq!(received, sent);

    for b in serialised {
        block!(tx.write(*b)).ok();
    }

    discovery_app::exit()
}

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    defmt::panic!()
}

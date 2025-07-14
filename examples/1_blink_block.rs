//! Basic blocking blinking LED example using CLINT delays.

#![no_std]
#![no_main]

extern crate panic_halt;
use hifive1::{
    clock,
    hal::{prelude::*, DeviceResources},
    sprintln, stdout, Led,
};

const STEP_MS: u32 = 1000; // Blinking step in milliseconds

#[riscv_rt::entry]
fn main() -> ! {
    let device_resources = DeviceResources::take().unwrap();
    let core_peripherals = device_resources.core_peripherals;
    let peripherals = device_resources.peripherals;
    let pins = device_resources.pins;

    // Configure clocks and UART for stdout
    let clocks = clock::configure(peripherals.PRCI, peripherals.AONCLK, 320.mhz().into());
    stdout::configure(
        peripherals.UART0,
        pins.pin17,
        pins.pin16,
        115_200.bps(),
        clocks,
    );

    // Configure LED pin as an inverted output
    let pin = pins.pin5;
    let mut led = pin.into_inverted_output();

    // Get the MTIMER peripheral from CLINT. This is used for blocking delays.
    let mut mtimer = core_peripherals.clint.mtimer();

    loop {
        Led::toggle(&mut led);
        sprintln!("LED toggled. New state: {}", led.is_on());
        mtimer.delay_ms(STEP_MS);
    }
}

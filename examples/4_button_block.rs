//! Demonstration on how to configure the GPIO9 interrupt on HiFive boards.

#![no_main]
#![no_std]

extern crate panic_halt;
use hifive1::{
    Led, clock,
    hal::{DeviceResources, prelude::*},
    pin, sprintln, stdout,
};

const STEP_MS: u32 = 1000; // Blinking step in milliseconds

#[riscv_rt::entry]
fn main() -> ! {
    let device_resources = DeviceResources::take().unwrap();
    let core_peripherals = device_resources.core_peripherals;
    let peripherals = device_resources.peripherals;
    let pins = device_resources.pins;

    // Configure clocks
    let clocks = clock::configure(peripherals.PRCI, peripherals.AONCLK, 320.mhz().into());

    // Configure UART for stdout
    stdout::configure(
        peripherals.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );

    // Configure GPIOs
    sprintln!("Configuring GPIOs...");
    let mut button = pins.pin9.into_pull_up_input();
    let mut led = pin!(pins, led_blue).into_inverted_output();

    // Get the MTIMER peripheral from CLINT. This is used for blocking delays.
    let mut mtimer = core_peripherals.clint.mtimer();

    loop {
        if button.is_low().unwrap() {
            led.on();
            sprintln!("Button pressed, LED is ON");
        } else {
            led.off();
            sprintln!("Button released, LED is OFF");
        }
        mtimer.delay_ms(STEP_MS);
    }
}

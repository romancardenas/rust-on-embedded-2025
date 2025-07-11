//! Basic asynchronous blinking LED example using CLINT.

#![no_std]
#![no_main]

extern crate panic_halt;
use hifive1::{
    Led, clock,
    hal::{DeviceResources, asynch::delay::Delay, asynch::prelude::*, prelude::*},
    pin, sprintln, stdout,
};

const STEP_MS: u32 = 1000; // Blinking step in milliseconds

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) -> ! {
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

    // Configure blue LED pin as an inverted output
    let pin = pin!(pins, led_blue);
    let mut led = pin.into_inverted_output();

    // Configure MTIMER interrupts to allow asynchronous delays
    let mtimer = core_peripherals.clint.mtimer();
    mtimer.disable();
    let (mtimecmp, mtime) = (mtimer.mtimecmp0(), mtimer.mtime());
    mtime.write(0);
    mtimecmp.write(u64::MAX);
    unsafe { riscv::interrupt::enable() };

    // Create an asynchronous delay instance
    let mut delay = Delay::new(mtimer);

    loop {
        Led::toggle(&mut led);
        sprintln!("LED toggled. New state: {}", led.is_on());
        delay.delay_ms(STEP_MS).await;
    }
}

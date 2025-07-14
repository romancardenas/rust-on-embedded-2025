//! Basic asynchronous blinking LED example.

#![no_std]
#![no_main]

extern crate panic_halt;
use hifive1::{
    clock,
    hal::{asynch::delay::Delay, asynch::prelude::*, prelude::*, DeviceResources},
    sprintln, stdout, Led,
};

const STEP_MS: u32 = 1000; // Blinking step in milliseconds

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) -> ! {
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

    // Configure blue LED pin as an inverted output
    let mut led = pins.pin5.into_inverted_output();

    // Configure MTIMER interrupts to allow asynchronous delays
    let mtimer = core_peripherals.clint.mtimer();
    let (mtimecmp, mtime) = (mtimer.mtimecmp0(), mtimer.mtime());
    mtime.write(0);
    mtimecmp.write(u64::MAX);
    // Create an asynchronous delay instance
    let mut delay = Delay::new(mtimer);

    // Enable interrupts
    unsafe { riscv::interrupt::enable() };

    loop {
        Led::toggle(&mut led);
        sprintln!("LED toggled. New state: {}", led.is_on());
        delay.delay_ms(STEP_MS).await;
    }
}

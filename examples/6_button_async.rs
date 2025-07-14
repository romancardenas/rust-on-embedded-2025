//! Asynchronous example that toggles an LED when a button is pressed.

#![no_std]
#![no_main]

extern crate panic_halt;
use hifive1::{
    clock,
    hal::{asynch::prelude::*, gpio::EventType, prelude::*, DeviceResources},
    sprintln, stdout, Led,
};

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

    // Configure GPIOs
    let mut button = pins.pin9.into_pull_up_input();
    let mut led = pins.pin5.into_inverted_output();

    // Configure interrupts
    button.disable_interrupt(EventType::All);
    button.clear_pending_interrupt(EventType::All);
    unsafe {
        button.enable_exti();
        button.set_exti_priority(Priority::P1);
    }

    // Enable GPIO9 interrupt in PLIC
    let plic = core_peripherals.plic;
    unsafe {
        plic.ctx0().threshold().set_threshold(Priority::P0);
        plic.enable();
        riscv::interrupt::enable();
    }

    loop {
        button.wait_for_high().await.unwrap();
        led.off();
        sprintln!("Button released, LED is OFF");

        button.wait_for_low().await.unwrap();
        led.on();
        sprintln!("Button pressed, LED is ON");
    }
}

//! Interrupt-driven example that toggles an LED when a button is pressed.

#![no_main]
#![no_std]

extern crate panic_halt;
use hifive1::{
    clock,
    hal::{e310x::Gpio0, gpio::EventType, prelude::*, DeviceResources},
    sprintln, stdout, Led,
};

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

    // Configure GPIOs
    let mut button = pins.pin9.into_pull_up_input();
    let mut led = pins.pin5.into_inverted_output();

    // Configure interrupts
    button.disable_interrupt(EventType::All);
    button.clear_pending_interrupt(EventType::All);
    button.enable_interrupt(EventType::BothEdges);
    unsafe {
        button.enable_exti();
        button.set_exti_priority(Priority::P1);
    }

    // Enable button interrupts for both edges, set priority, and enable PLIC
    sprintln!("Enabling external interrupts...");
    let plic = core_peripherals.plic;
    unsafe {
        plic.ctx0().threshold().set_threshold(Priority::P0);
        plic.enable();
        riscv::interrupt::enable();
    }

    loop {
        if button.is_low().unwrap() {
            led.on();
            sprintln!("Button pressed, LED is ON");
        } else {
            led.off();
            sprintln!("Button released, LED is OFF");
        }
        riscv::asm::wfi();
    }
}

/// Handler for the GPIO9 interrupt
#[riscv_rt::external_interrupt(ExternalInterrupt::GPIO9)]
fn gpio9_handler() {
    sprintln!("    --- GPIO9 interrupt!");
    // Clear the GPIO pending interrupt
    let gpio_block = unsafe { Gpio0::steal() };
    gpio_block.fall_ip().write(|w| w.pin9().set_bit());
    gpio_block.rise_ip().write(|w| w.pin9().set_bit());
}

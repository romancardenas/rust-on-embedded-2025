//! Demonstration on how to configure the GPIO9 interrupt on HiFive boards.

#![no_main]
#![no_std]

extern crate panic_halt;
use hifive1::{
    clock,
    hal::{e310x::Gpio0, gpio::EventType, prelude::*, DeviceResources},
    pin, sprintln, stdout, Led,
};

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
    // Configure button pin (GPIO9) as pull-up input
    let mut button = pins.pin9.into_pull_up_input();
    // Configure blue LED pin (GPIO21) as inverted output
    let mut led = pin!(pins, led_blue).into_inverted_output();

    // Configure interrupts
    sprintln!("Configuring interrupts...");
    let plic = core_peripherals.plic;

    // Make sure that interrupts are cleared before configuring it
    let priorities = plic.priorities();
    priorities.reset::<ExternalInterrupt>();
    let ctx = plic.ctx0();
    ctx.enables().disable_all::<ExternalInterrupt>();
    button.disable_interrupt(EventType::All);
    button.clear_pending_interrupt(EventType::All);

    // Enable button interrupts for both edges, set priority, and enable PLIC
    sprintln!("Enabling external interrupts...");
    button.enable_interrupt(EventType::BothEdges);
    unsafe {
        button.enable_exti();
        button.set_exti_priority(Priority::P1);
        ctx.threshold().set_threshold(Priority::P0);
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
        sprintln!();
        riscv::asm::wfi();
    }
}

/// Handler for the GPIO9 interrupt
#[riscv_rt::external_interrupt(ExternalInterrupt::GPIO9)]
fn gpio9_handler() {
    sprintln!("--- GPIO9 interrupt!");
    // Clear the GPIO pending interrupt
    let gpio_block = unsafe { Gpio0::steal() };
    gpio_block.fall_ip().write(|w| w.pin9().set_bit());
    gpio_block.rise_ip().write(|w| w.pin9().set_bit());
}

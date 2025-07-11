//! Basic blinking LEDs example using mtime/mtimecmp registers for "sleep" in a loop.
//! Blinks each led once and goes to the next one.
//! This example uses synchronous UART and only tests asynchronous Delay.

#![no_std]
#![no_main]

extern crate panic_halt;
use hifive1::{
    clock,
    hal::{asynch::prelude::*, e310x::Gpio0, gpio::EventType, prelude::*, DeviceResources},
    pin, sprintln, Led,
};

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) -> ! {
    let dr = DeviceResources::take().unwrap();
    let cp = dr.core_peripherals;
    let p = dr.peripherals;
    let pins: hifive1::hal::device::DeviceGpioPins = dr.pins;

    // Configure clocks
    let clocks = clock::configure(p.PRCI, p.AONCLK, 320.mhz().into());

    //Blinking LED
    let pin = pin!(pins, led_blue);
    let mut led = pin.into_inverted_output();

    // Configure UART for stdout
    hifive1::stdout::configure(
        p.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );

    // Button pin (GPIO9) as pull-up input
    let mut button = pins.pin9.into_pull_up_input();

    // Set button interrupt source priority
    let plic = cp.plic;
    let priorities = plic.priorities();
    priorities.reset::<ExternalInterrupt>();
    unsafe { button.set_exti_priority(Priority::P1) };

    // Clear pending interrupts from previous states
    Gpio0::disable_interrupts(EventType::All);
    Gpio0::clear_pending_interrupts(EventType::All);

    // Enable GPIO9 interrupt in PLIC
    let ctx = plic.ctx0();
    unsafe {
        ctx.enables().disable_all::<ExternalInterrupt>();
        ctx.threshold().set_threshold(Priority::P0);
        button.enable_exti();
        plic.enable();
        riscv::interrupt::enable();
    };

    // Execute loop
    loop {
        button.wait_for_high().await.unwrap();
        led.off();
        sprintln!("Button released, LED is OFF");

        button.wait_for_low().await.unwrap();
        led.on();
        sprintln!("Button pressed, LED is ON");
    }
}

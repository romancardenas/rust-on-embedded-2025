//! Basic blocking blinking LED example using CLINT interrupts.

#![no_main]
#![no_std]

extern crate panic_halt;
use hifive1::{
    clock,
    hal::{e310x::Clint, prelude::*, DeviceResources},
    pin, sprintln, stdout, Led,
};

const PERIOD_MS: u64 = 1000;
const MTIMER_FREQUENCY_HZ: u64 = 32768;
const CLINT_TICKS_PER_PERIOD: u64 = PERIOD_MS * MTIMER_FREQUENCY_HZ / 1000;

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

    // Configure blue LED pin as an inverted output
    let pin = pin!(pins, led_blue);
    let mut led = pin.into_inverted_output();

    // Configure the MTIMER peripheral from CLINT. This is used for interrupt-driven delays.
    sprintln!("Configuring CLINT's MTIMER...");
    let mtimer = core_peripherals.clint.mtimer();
    let (mtimecmp, mtime) = (mtimer.mtimecmp_mhartid(), mtimer.mtime());
    mtime.write(0);
    mtimecmp.write(CLINT_TICKS_PER_PERIOD); // Set initial mtimecmp to schedule the first interrupt

    sprintln!("Enabling interrupts...");
    unsafe {
        mtimer.enable(); // Enable the MTIMER interrupt
        riscv::interrupt::enable(); // Enable global interrupts
    }

    loop {
        Led::toggle(&mut led);
        sprintln!("LED toggled. New state: {}", led.is_on());
        sprintln!();
        riscv::asm::wfi();
    }
}

/// Handler for the machine timer interrupt (handled by the CLINT)
#[riscv_rt::core_interrupt(CoreInterrupt::MachineTimer)]
fn mtimer_handler() {
    sprintln!("--- MTIMER interrupt!");
    // SAFETY: Interrupt triggered by the CLINT, we can safely access it
    let clint = unsafe { Clint::steal() };
    let mtimer = clint.mtimer();
    mtimer
        .mtimecmp_mhartid()
        .modify(|f| *f += CLINT_TICKS_PER_PERIOD);
}

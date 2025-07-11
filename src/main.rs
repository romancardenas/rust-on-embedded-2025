//! Prints "Hello, world!" via the host console using the UART0 peripheral.

#![no_std] // Rust no_std environment (no standard library)
#![no_main] // No main function, using riscv_rt for entry point

extern crate panic_halt; // Use panic_halt to halt the program on panic
use hifive1::{
    clock,
    hal::{prelude::*, DeviceResources},
    pin, sprintln, stdout,
};

// The riscv_rt::entry attribute marks the main function as the entry point for the program.
#[riscv_rt::entry]
fn main() -> ! {
    // Take device resources. This is a singleton, and only one instance should be created.
    let device_resources = DeviceResources::take().unwrap();

    // Device resources are classified into three categories:
    // - `core_peripherals`: Core peripherals like CLINT and PLIC.
    // - `peripherals`: Platform peripherals like PRCI, AONCLK, UART0. GPIO0 is not included here
    // - `pins`: GPIO pins for the device.
    let _core_peripherals = device_resources.core_peripherals;
    let peripherals = device_resources.peripherals;
    let pins = device_resources.pins;

    // Usually, before the main loop, we configure the system clocks...
    let prci = peripherals.PRCI;
    let aonclk = peripherals.AONCLK;
    let clocks = clock::configure(prci, aonclk, 320.mhz().into());
    // ... and configure UART for stdout
    let uart0 = peripherals.UART0;
    let uart0_tx = pin!(pins, uart0_tx);
    let uart0_rx = pin!(pins, uart0_rx);
    stdout::configure(uart0, uart0_tx, uart0_rx, 115_200.bps(), clocks);

    // Now we can print to the host console
    sprintln!("Hello, world!");
    loop {
        riscv::asm::wfi(); // Go to sleep recursively
    }
}

#![no_main]
#![no_std]

extern crate panic_halt;
use hifive1::hal::e310x;
use riscv_rt as _;

#[rtic::app(device = e310x)]
mod app {
    use super::e310x;
    use hifive1::{
        clock,
        hal::{
            asynch::{delay::Delay, prelude::*},
            gpio::{
                gpio0::{Pin5, Pin9},
                EventType, Input, Invert, Output, PullUp, Regular,
            },
            prelude::*,
            DeviceResources,
        },
        pin, sprintln, stdout, Led,
    };

    pub type Button = Pin9<Input<PullUp>>;
    pub type BlueLed = Pin5<Output<Regular<Invert>>>;

    #[shared]
    struct Shared {
        pressed: bool,
    }

    #[local]
    struct Local {
        button: Button,
        delay: Delay,
        led: BlueLed,
    }

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        let device_resources = unsafe { DeviceResources::steal() };
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

        sprintln!("[init] Configuring GPIOs...");
        // Configure button pin (GPIO9) as pull-up input
        let button = pins.pin9.into_pull_up_input();
        // Configure blue LED pin (GPIO21) as inverted output
        let led = pins.pin5.into_inverted_output();

        // Configure MTIMER interrupts to allow asynchronous delays
        sprintln!("[init] Configuring MTIMER for asynchronous delays...");
        let mtimer = core_peripherals.clint.mtimer();
        mtimer.disable();
        let (mtimecmp, mtime) = (mtimer.mtimecmp0(), mtimer.mtime());
        mtime.write(0);
        mtimecmp.write(u64::MAX);
        let delay = Delay::new(mtimer);

        // Configure interrupts for button
        sprintln!("[init] Configuring external interrupts..."); // Set button interrupt source priority
        let plic = core_peripherals.plic;
        let priorities = plic.priorities();
        priorities.reset::<ExternalInterrupt>();
        unsafe { button.set_exti_priority(Priority::P1) };

        // Enable GPIO9 interrupt in PLIC
        button.enable_interrupt(EventType::BothEdges);
        let ctx = plic.ctx0();
        unsafe {
            ctx.enables().disable_all::<ExternalInterrupt>();
            ctx.threshold().set_threshold(Priority::P0);
            button.enable_exti();
            plic.enable();
        };

        (Shared { pressed: false }, Local { button, delay, led })
    }

    #[idle]
    fn idle(mut _cx: idle::Context) -> ! {
        button_task::spawn().unwrap(); // Start button task
        loop {
            sprintln!("    [idle] Waiting for interrupts");
            riscv::asm::wfi();
        }
    }

    #[task(shared = [pressed], local = [button, delay], priority = 1)]
    async fn button_task(mut cx: button_task::Context) {
        let button = cx.local.button;
        let delay = cx.local.delay;

        // Make sure that button is released before starting the task
        button.wait_for_high().await.unwrap();
        sprintln!("        [button_task]: Started");

        loop {
            button.wait_for_any_edge().await.unwrap(); // async wait for button state change
            sprintln!("        [button_task]: Button state changed");
            delay.delay_ms(100).await; // async debounce delay

            let pressed = button.is_low().unwrap();
            sprintln!("        [button_task]: Button pressed: {}", pressed);

            // Update shared state and spawn led_task
            cx.shared.pressed.lock(|shared_pressed| {
                *shared_pressed = pressed;
            });
            led_task::spawn().unwrap();

            sprintln!("        [button_task]: iteration finished")
        }
    }

    #[task(shared = [pressed], local = [led], priority = 2)]
    async fn led_task(mut cx: led_task::Context) {
        let led = cx.local.led;
        let pressed = cx.shared.pressed.lock(|shared_pressed| *shared_pressed);
        if pressed {
            led.on();
            sprintln!("            [led_task]: LED is ON");
        } else {
            led.off();
            sprintln!("            [led_task]: LED is OFF");
        }
    }
}

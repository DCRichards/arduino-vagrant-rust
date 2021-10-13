#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

extern crate alloc;
extern crate arduino_nano33iot as bsp;

use {
    alloc_cortex_m::CortexMHeap,
    bsp::entry,
    bsp::hal::delay::Delay,
    bsp::hal::pac::{CorePeripherals, Peripherals},
    bsp::hal::prelude::*,
    bsp::hal::clock::GenericClockController,
    bsp::Pins,
    panic_halt as _,
};

mod sensors;
mod usb_logger;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    unsafe {
        ALLOCATOR.init(cortex_m_rt::heap_start() as usize, 1024);
    };

    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let pins = Pins::new(peripherals.PORT);
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let logger = usb_logger::USBLogger::new(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.PM,
        pins.usb_dm.into(),
        pins.usb_dp.into(),
        &mut core.NVIC,
    );

    let mut led: bsp::Led = pins.led_sck.into();
    let mut temperature_pin = pins.d2.into_readable_output();
    let moisture_pin = pins.a0.into_alternate();

    let mut moisture_sensor = sensors::Moisture::new(
        moisture_pin,
        peripherals.ADC,
        &mut peripherals.PM,
        &mut clocks,
    );
    let mut temperature_sensor =
        sensors::Temperature::new(&mut temperature_pin, &mut delay).unwrap();

    led.set_high().unwrap();

    loop {
        let temperature = temperature_sensor.read().unwrap();
        let moisture = moisture_sensor.read().unwrap();
        logger.log(
            alloc::format!(
                "Temperature: {}°C\r\nMoisture: {}\r\n",
                temperature,
                moisture,
            )
            .as_bytes(),
        );
    }
}

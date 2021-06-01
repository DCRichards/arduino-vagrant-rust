#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

extern crate alloc;
extern crate arduino_nano33iot as hal;

use {
    hal::adc::Adc,
    hal::clock::GenericClockController,
    hal::delay::Delay,
    hal::entry,
    hal::pac::{CorePeripherals, Peripherals},
    hal::prelude::*,
};

use alloc_cortex_m::CortexMHeap;

mod temperature;
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
    let mut pins = hal::Pins::new(peripherals.PORT);
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
        pins.usb_dm,
        pins.usb_dp,
        &mut pins.port,
        &mut core.NVIC,
    );

    // let mut led = pins.led_sck.into_open_drain_output(&mut pins.port);
    let mut temp_sensor = pins.d2.into_readable_open_drain_output(&mut pins.port);
    let mut adc = Adc::adc(peripherals.ADC, &mut peripherals.PM, &mut clocks);
    // adc.reference(hal::pac::adc::refctrl::REFSEL_A::AREFA);
    let mut moisture_sensor = pins.a0.into_function_b(&mut pins.port);

    let mut temperature_sensor = temperature::TemperatureSensor::new(&mut temp_sensor, &mut delay);

    loop {
        let current_temp = temperature_sensor.read();
        let current_moisture: u16 = adc.read(&mut moisture_sensor).unwrap();
        logger.log(
            alloc::format!(
                "Temperature: {}°C\r\nMoisture: {}\r\n",
                current_temp,
                current_moisture
            )
            .as_bytes(),
        );
    }
}

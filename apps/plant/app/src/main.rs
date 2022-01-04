#![no_std]
#![no_main]

extern crate arduino_nano33iot as bsp;

use {
    bsp::entry,
    bsp::hal::adc::Adc,
    bsp::hal::clock::{ClockGenId, ClockSource, GenericClockController},
    bsp::hal::delay::Delay,
    bsp::hal::pac::{CorePeripherals, Peripherals},
    bsp::hal::prelude::*,
    bsp::hal::rtc,
    bsp::hal::time::KiloHertz,
    bsp::Pins,
    core::fmt::Write,
    panic_halt as _,
};

mod display;
mod sensors;
mod usb_logger;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let pins = Pins::new(peripherals.PORT);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let logger = usb_logger::USBLogger::new(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.PM,
        pins.usb_dm.into(),
        pins.usb_dp.into(),
        &mut core.NVIC,
    );

    let timer_clock = clocks
        .configure_gclk_divider_and_source(ClockGenId::GCLK3, 32, ClockSource::OSC32K, true)
        .unwrap();
    let rtc_clock = clocks.rtc(&timer_clock).unwrap();
    let rtc = rtc::Rtc::clock_mode(peripherals.RTC, rtc_clock.freq(), &mut peripherals.PM);

    let mut adc = Adc::adc(peripherals.ADC, &mut peripherals.PM, &mut clocks);
    // Nano 33 IOT has a 10bit resolution
    adc.resolution(bsp::pac::adc::ctrlb::RESSEL_A::_10BIT);
    // AREFA = AREF pin according to
    // https://ww1.microchip.com/downloads/en/DeviceDoc/SAM-D21DA1-Family-Data-Sheet-DS40001882G.pdf#_OPENTOPIC_TOC_PROCESSING_d10240e23103
    adc.reference(bsp::pac::adc::refctrl::REFSEL_A::AREFA);
    // Override default of 1/2.
    adc.gain(bsp::pac::adc::inputctrl::GAIN_A::_1X);

    let i2c = bsp::i2c_master(
        &mut clocks,
        KiloHertz(400),
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sda,
        pins.scl,
    );

    let oled_display = display::Display::new(i2c);
    let mut oled_display = match oled_display {
        Ok(disp) => disp,
        Err(e) => {
            if let Some(s) = format_args!("{:?}", e).as_str() {
                logger.log(s);
            };
            panic!("{:?}", e);
        }
    };

    oled_display.clear().unwrap();
    write!(oled_display, "Loading...").unwrap();

    // Pins
    let mut led: bsp::Led = pins.led_sck.into();
    let mut temperature_pin = pins.d2.into_readable_output();
    let moisture_pin = pins.a0.into_alternate();
    let ldr_pin = pins.a1.into_alternate();

    let temperature_sensor = sensors::Temperature::new(&mut temperature_pin, &mut delay);
    let mut temperature_sensor = match temperature_sensor {
        Ok(ts) => ts,
        Err(e) => {
            if let Some(s) = format_args!("{:?}", e).as_str() {
                logger.log(s);
            };
            panic!("{:?}", e);
        }
    };

    let mut soil_moisture_sensor = sensors::SoilMoisture::new(moisture_pin);
    let mut light_sensor = sensors::Light::new(ldr_pin);

    // Light LED to indicate all OK
    led.set_high().unwrap();

    loop {
        let time = rtc.current_time();
        let temperature = temperature_sensor.read().unwrap();
        let moisture = soil_moisture_sensor.read(&mut adc).unwrap();
        let light = light_sensor.read(&mut adc).unwrap();

        oled_display.clear().unwrap();
        write!(
            oled_display,
            "{:.2}°C\nMoisture: {}\nLight:    {}\n{:02}:{:02}:{:02}",
            temperature, moisture, light, time.hours, time.minutes, time.seconds,
        )
        .unwrap();
    }
}

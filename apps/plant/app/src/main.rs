#![no_std]
#![no_main]

extern crate arduino_nano33iot as bsp;

use {
    bsp::entry,
    bsp::hal::adc::Adc,
    bsp::hal::clock::{ClockGenId, ClockSource, GenericClockController},
    bsp::hal::delay::Delay,
    bsp::hal::eic::pin::{ExtInt11, Sense},
    bsp::hal::eic::EIC,
    bsp::hal::gpio::v2::pin::PB11,
    bsp::hal::gpio::v2::{Pin, PullUpInterrupt},
    bsp::hal::pac::{CorePeripherals, Peripherals},
    bsp::hal::prelude::*,
    bsp::hal::rtc,
    bsp::hal::time::KiloHertz,
    bsp::pac::interrupt,
    bsp::Pins,
    core::fmt::Write,
    cortex_m::interrupt::free,
    cortex_m::peripheral::NVIC,
    lib::percent_from_adc_reading,
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

    // Real-time clock
    let timer_clock = clocks
        .configure_gclk_divider_and_source(ClockGenId::GCLK3, 32, ClockSource::OSC32K, true)
        .unwrap();
    let rtc_clock = clocks.rtc(&timer_clock).unwrap();
    let rtc = rtc::Rtc::clock_mode(peripherals.RTC, rtc_clock.freq(), &mut peripherals.PM);

    // Analogue → Digital Converter
    let mut adc = Adc::adc(peripherals.ADC, &mut peripherals.PM, &mut clocks);
    // Nano 33 IOT has a 10bit resolution (0 to 1023)
    adc.resolution(bsp::hal::adc::Resolution::_10BIT);
    // Set reference to AREF pin (~3.3V) and 1x gain, matching analogReference(AR_EXTERNAL)
    // https://github.com/arduino/ArduinoCore-samd/blob/master/cores/arduino/wiring_analog.c#L105
    adc.reference(bsp::hal::adc::Reference::AREFA);
    adc.gain(bsp::hal::adc::Gain::_1X);
    adc.prescaler(bsp::hal::adc::Prescaler::DIV512);

    // External Interrupt
    let gclk0 = clocks.gclk0();
    let eic_clock = clocks.eic(&gclk0).unwrap();
    let mut eic = EIC::init(&mut peripherals.PM, eic_clock, peripherals.EIC);

    // I2C bus
    let i2c = bsp::i2c_master(
        &mut clocks,
        KiloHertz(400),
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sda,
        pins.scl,
    );

    // Display
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
    let button_pin: Pin<PB11, PullUpInterrupt> = pins.d3.into();

    // Init sensors
    let mut soil_moisture_sensor = sensors::SoilMoisture::new(moisture_pin);
    let mut light_sensor = sensors::Light::new(ldr_pin);
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

    // Init buttons and interrupt
    //
    // A mapping of Pin to interrupt can be found here
    // https://github.com/atsamd-rs/atsamd/blob/master/hal/src/thumbv6m/eic/pin.rs#L238
    let mut button_extint = ExtInt11::new(button_pin);
    button_extint.sense(&mut eic, Sense::FALL);
    button_extint.enable_interrupt(&mut eic);
    unsafe {
        core.NVIC.set_priority(interrupt::EIC, 1);
        // Enable the interrupt in the NVIC.
        NVIC::unmask(interrupt::EIC);
    }

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
            "{:.2}°C\nMoist: {:.1}%\nLight: {:.1}%\n{:02}:{:02}:{:02}",
            temperature,
            100.0 - percent_from_adc_reading(10, moisture),
            percent_from_adc_reading(10, light),
            time.hours,
            time.minutes,
            time.seconds,
        )
        .unwrap();
    }
}

#[interrupt]
fn EIC() {
    // See SAM-D21DA1-Family-Data-Sheet-DS40001882G.pdf §16.6.5 for more
    // information on interrupt registers.
    let eic = unsafe { &*bsp::pac::EIC::ptr() };

    free(|_| {
        // The interrupt flag in the INTFLAG register is set when the interrupt
        // condition occurs - so check for it here in the case of multiple extints.
        if eic.intflag.read().extint11().bit_is_set() {
            // The interrupt request remains active until the interrupt flag is cleared,
            // the interrupt is disabled or the peripheral is reset. Clear by writing a
            // 1 to the corresponding bit in the INTFLAG register.
            eic.intflag.modify(|_, w| w.extint11().set_bit());
        }
    });
}

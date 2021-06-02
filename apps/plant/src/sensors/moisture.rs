use {
    hal::adc::Adc,
    hal::clock::GenericClockController,
    hal::gpio::v2::{Alternate, B, PA02},
    hal::gpio::Pin,
    hal::pac::{ADC, PM},
    hal::prelude::_embedded_hal_adc_OneShot,
};

/// Represents a physical soil moisture sensor.
pub struct MoistureSensor {
    adc: Adc<ADC>,
    pin: Pin<PA02, Alternate<B>>,
}

impl MoistureSensor {
    /// Initialise the sensor.
    pub fn new(
        pin: Pin<PA02, Alternate<B>>,
        adc_peripheral: ADC,
        pm_peripheral: &mut PM,
        clocks: &mut GenericClockController,
    ) -> Self {
        MoistureSensor {
            adc: Adc::adc(adc_peripheral, pm_peripheral, clocks),
            pin: pin,
        }
    }

    /// Reads the current moisture level.
    pub fn read(&mut self) -> u16 {
        // adc.reference(hal::pac::adc::refctrl::REFSEL_A::AREFB);
        self.adc.read(&mut self.pin).unwrap()
    }
}

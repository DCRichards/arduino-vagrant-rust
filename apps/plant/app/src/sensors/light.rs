use {
    super::error::SensorError,
    super::Result,
    bsp::hal::adc::Adc,
    bsp::hal::gpio::v2::pin::{Pin, PB02},
    bsp::hal::gpio::v2::{Alternate, B},
    bsp::hal::prelude::_embedded_hal_adc_OneShot,
    bsp::pac::ADC,
};

/// Represents a physical light sensor.
pub struct Light {
    pin: Pin<PB02, Alternate<B>>,
}

impl Light {
    /// Initialise the light sensor.
    pub fn new(pin: Pin<PB02, Alternate<B>>) -> Self {
        Light { pin }
    }

    /// Reads the current light level.
    pub fn read(&mut self, adc: &mut Adc<ADC>) -> Result<u16> {
        match adc.read(&mut self.pin) {
            Ok(val) => Ok(val),
            Err(_) => Err(SensorError::ReadError),
        }
    }
}

use {
    super::error::SensorError,
    super::Result,
    bsp::hal::adc::Adc,
    bsp::hal::gpio::v2::pin::{Pin, PA02},
    bsp::hal::gpio::v2::{Alternate, B},
    bsp::hal::prelude::_embedded_hal_adc_OneShot,
    bsp::pac::ADC,
};

/// Represents a physical soil moisture sensor.
pub struct SoilMoisture {
    pin: Pin<PA02, Alternate<B>>,
}

impl SoilMoisture {
    // Initialise the soil moisture sensor.
    pub fn new(pin: Pin<PA02, Alternate<B>>) -> Self {
        SoilMoisture { pin }
    }

    /// Reads the current moisture level.
    pub fn read(&mut self, adc: &mut Adc<ADC>) -> Result<u16> {
        match adc.read(&mut self.pin) {
            Ok(val) => Ok(val),
            Err(_) => Err(SensorError::ReadError),
        }
    }
}

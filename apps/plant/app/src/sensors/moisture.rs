use {
    super::error::SensorError,
    super::Result,
    bsp::hal::adc::Adc,
    bsp::hal::clock::GenericClockController,
    bsp::hal::gpio::v2::pin::{Pin, PA02},
    bsp::hal::gpio::v2::{Alternate, B},
    bsp::hal::prelude::_embedded_hal_adc_OneShot,
    bsp::pac::{ADC, PM},
};

/// Represents a physical soil moisture sensor.
pub struct Moisture {
    adc: Adc<ADC>,
    pin: Pin<PA02, Alternate<B>>,
}

impl Moisture {
    /// Initialise the sensor.
    pub fn new(
        pin: Pin<PA02, Alternate<B>>,
        adc_peripheral: ADC,
        pm_peripheral: &mut PM,
        clocks: &mut GenericClockController,
    ) -> Self {
        let mut adc = Adc::adc(adc_peripheral, pm_peripheral, clocks);
        // AREFA = AREF pin according to
        // https://ww1.microchip.com/downloads/en/DeviceDoc/SAM-D21DA1-Family-Data-Sheet-DS40001882G.pdf#_OPENTOPIC_TOC_PROCESSING_d10240e23103
        adc.reference(bsp::pac::adc::refctrl::REFSEL_A::AREFA);
        // Set the gain to be 1x, as the default is /2.
        adc.gain(bsp::pac::adc::inputctrl::GAIN_A::_1X);

        Moisture { adc: adc, pin: pin }
    }

    /// Reads the current moisture level.
    pub fn read(&mut self) -> Result<u16> {
        return match self.adc.read(&mut self.pin) {
            Ok(val) => Ok(val),
            Err(_) => Err(SensorError::ReadError),
        };
    }
}

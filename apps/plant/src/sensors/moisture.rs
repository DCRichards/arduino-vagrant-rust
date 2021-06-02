use {
    hal::adc::Adc,
    hal::clock::GenericClockController,
    hal::gpio::v2::{Alternate, B, PA02},
    hal::gpio::Pin,
    hal::pac::{ADC, PM},
    hal::prelude::_embedded_hal_adc_OneShot,
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
        adc.reference(hal::pac::adc::refctrl::REFSEL_A::AREFA);
        // Set the gain to be 1x, as the default is /2.
        adc.gain(hal::pac::adc::inputctrl::GAIN_A::_1X);

        Moisture { adc: adc, pin: pin }
    }

    /// Reads the current moisture level.
    pub fn read(&mut self) -> u16 {
        // adc.reference(hal::pac::adc::refctrl::REFSEL_A::AREFB);
        self.adc.read(&mut self.pin).unwrap()
    }
}
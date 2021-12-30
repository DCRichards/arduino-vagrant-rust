use {
    super::error::SensorError,
    super::Result,
    bsp::hal::delay::Delay,
    bsp::hal::gpio::v2::pin::{Pin, ReadableOutput, PB10},
    bsp::hal::prelude::_embedded_hal_blocking_delay_DelayMs,
    lib::temp_from_raw,
    onewire::ds18b20::DS18B20,
    onewire::{DeviceSearch, OneWire},
};

/// Represents a physical temperature sensor.
pub struct Temperature<'a> {
    delay: &'a mut Delay,
    one_wire: OneWire<'a, core::convert::Infallible>,
    sensor: DS18B20,
}

impl<'a> Temperature<'a> {
    /// Initialise the sensor.
    pub fn new(pin: &'a mut Pin<PB10, ReadableOutput>, delay: &'a mut Delay) -> Result<Self> {
        let mut one_wire = OneWire::new(pin, false);
        one_wire.reset(delay)?;

        let mut search = DeviceSearch::new_for_family(onewire::ds18b20::FAMILY_CODE);
        let sensor = match one_wire.search_next(&mut search, delay).unwrap() {
            None => return Err(SensorError::NoDevice),
            Some(device) => DS18B20::new::<()>(device)?,
        };

        Ok(Temperature {
            delay,
            one_wire,
            sensor,
        })
    }

    /// Reads the current temperature.
    pub fn read(&mut self) -> Result<f32> {
        let resolution = self
            .sensor
            .measure_temperature(&mut self.one_wire, self.delay)?;
        self.delay.delay_ms(resolution.time_ms());
        let raw = self
            .sensor
            .read_temperature(&mut self.one_wire, self.delay)?;

        Ok(temp_from_raw(raw))
    }
}

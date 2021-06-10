use {
    super::error::SensorError,
    super::Result,
    hal::delay::Delay,
    hal::gpio::v2::{Output, Readable, PB10},
    hal::gpio::Pin,
    hal::prelude::_embedded_hal_blocking_delay_DelayMs,
    onewire::ds18b20::{split_temp, DS18B20},
    onewire::{DeviceSearch, OneWire},
};

/// Represents a physical temperature sensor.
pub struct Temperature<'a> {
    delay: &'a mut Delay,
    one_wire: OneWire<'a, ()>,
    sensor: DS18B20,
}

impl<'a> Temperature<'a> {
    /// Initialise the sensor.
    pub fn new(pin: &'a mut Pin<PB10, Output<Readable>>, delay: &'a mut Delay) -> Result<Self> {
        let mut one_wire = OneWire::new(pin, false);
        one_wire.reset(delay)?;

        let mut search = DeviceSearch::new_for_family(onewire::ds18b20::FAMILY_CODE);
        let sensor = match one_wire.search_next(&mut search, delay).unwrap() {
            None => return Err(SensorError::NoDevice),
            Some(device) => DS18B20::new::<()>(device)?,
        };

        Ok(Temperature {
            delay: delay,
            one_wire: one_wire,
            sensor: sensor,
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
        let (integer, fraction) = split_temp(raw);

        Ok(integer as f32 + fraction as f32 / 10000_f32)
    }
}

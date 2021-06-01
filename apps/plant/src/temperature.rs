use {
    core::convert::Infallible,
    hal::delay::Delay,
    hal::prelude::_embedded_hal_blocking_delay_DelayMs,
    onewire::ds18b20::*,
    onewire::{DeviceSearch, OneWire, OpenDrainOutput},
};

pub struct TemperatureSensor<'a> {
    delay: &'a mut Delay,
    one_wire: OneWire<'a, ()>,
    sensor: DS18B20,
}

impl<'a> TemperatureSensor<'a> {
    pub fn new(pin: &'a mut dyn OpenDrainOutput<()>, delay: &'a mut Delay) -> Self {
        let mut one_wire = OneWire::new(pin, false);
        one_wire.reset(delay).unwrap();
        let mut search = DeviceSearch::new_for_family(FAMILY_CODE);
        let sensor = match one_wire.search_next(&mut search, delay).unwrap() {
            None => panic!("No device found"),
            Some(device) => DS18B20::new::<Infallible>(device).unwrap(),
        };

        TemperatureSensor {
            delay: delay,
            one_wire: one_wire,
            sensor: sensor,
        }
    }

    pub fn read(&mut self) -> f32 {
        let resolution = self
            .sensor
            .measure_temperature(&mut self.one_wire, self.delay)
            .unwrap();
        self.delay.delay_ms(resolution.time_ms());
        let raw = self
            .sensor
            .read_temperature(&mut self.one_wire, self.delay)
            .unwrap();
        let (integer, fraction) = split_temp(raw);

        integer as f32 + fraction as f32 / 10000_f32
    }
}

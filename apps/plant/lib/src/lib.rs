#![cfg_attr(not(test), no_std)]
use onewire::ds18b20::split_temp;

/// Calculate a temperature reading from a raw value.
pub fn temp_from_raw(raw: u16) -> f32 {
    let (integer, fraction) = split_temp(raw);
    integer as f32 + fraction as f32 / 10000_f32
}

/// Calculate a percentage from an ADC reading of the given resolution.
pub fn percent_from_adc_reading(resolution: u32, reading: u16) -> f32 {
    let p = i32::pow(2, resolution) as f32 - 1.0;
    reading as f32 / p * 100.0
}

#[cfg(test)]
mod tests {
    use super::percent_from_adc_reading;
    use super::temp_from_raw;

    #[test]
    fn test_percent_from_adc_reading() {
        assert_eq!(percent_from_adc_reading(10, 1023), 100.0);
        assert_eq!(percent_from_adc_reading(10, 0), 0.0);
        assert_eq!(percent_from_adc_reading(10, 512), 50.048874);
        assert_eq!(percent_from_adc_reading(12, 2048), 50.012215);
    }

    #[test]
    fn test_temp_from_raw() {
        assert_eq!(temp_from_raw(0x00a2), 10.125);
        assert_eq!(temp_from_raw(0x0000), 0.0);
        assert_eq!(temp_from_raw(0xff00), -16.0);
    }
}

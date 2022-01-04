#![cfg_attr(not(test), no_std)]
use onewire::ds18b20::split_temp;

/// Calculate a temperature reading from a raw value.
pub fn temp_from_raw(raw: u16) -> f32 {
    let (integer, fraction) = split_temp(raw);
    integer as f32 + fraction as f32 / 10000_f32
}

#[cfg(test)]
mod tests {
    use super::temp_from_raw;

    #[test]
    fn test_temp_from_raw() {
        assert_eq!(temp_from_raw(0x00a2), 10.125);
        assert_eq!(temp_from_raw(0x0000), 0.0);
        assert_eq!(temp_from_raw(0xff00), -16.0);
    }
}

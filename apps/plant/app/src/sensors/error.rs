/// Errors whilst interacting with sensors
#[derive(Debug)]
pub enum SensorError {
    /// Error interacting with onewire bus
    OneWireError(onewire::Error<()>),
    /// Expected device not found
    NoDevice,
    /// Error obtaining reading from sensor
    ReadError,
}

impl From<onewire::Error<()>> for SensorError {
    fn from(err: onewire::Error<()>) -> Self {
        SensorError::OneWireError(err)
    }
}

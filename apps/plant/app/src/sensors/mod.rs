//! External sensors

pub mod error;
pub type Result<T> = core::result::Result<T, error::SensorError>;

mod temperature;
pub use temperature::Temperature;

mod moisture;
pub use moisture::Moisture;

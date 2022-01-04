//! External sensors

pub mod error;
pub type Result<T> = core::result::Result<T, error::SensorError>;

mod temperature;
pub use temperature::Temperature;

mod soil_moisture;
pub use soil_moisture::SoilMoisture;

mod light;
pub use light::Light;

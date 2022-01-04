use {
    bsp::hal::gpio::v2::pin::{Alternate, Pin, D, PB08, PB09},
    bsp::hal::sercom::v1::I2CMaster4,
    core::fmt::{Error, Write},
    ssd1306::mode::{TerminalMode, TerminalModeError},
    ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306},
};

#[derive(Debug)]
pub enum DisplayError {
    /// Display not initialised
    Uninitialized,
    /// Attempting to draw outside display bounds
    DrawOutOfBounds,
    /// An underlying driver error
    DriverError,
}

impl From<TerminalModeError> for DisplayError {
    fn from(e: TerminalModeError) -> Self {
        match e {
            TerminalModeError::Uninitialized => DisplayError::Uninitialized,
            TerminalModeError::OutOfBounds => DisplayError::DrawOutOfBounds,
            TerminalModeError::InterfaceError(_) => DisplayError::DriverError,
        }
    }
}

pub type Result<T> = core::result::Result<T, DisplayError>;

/// Represents a physical display
pub struct Display {
    driver: Ssd1306<I2CInterface<I2C>, DisplaySize128x32, TerminalMode>,
}

type I2C = I2CMaster4<Pin<PB08, Alternate<D>>, Pin<PB09, Alternate<D>>>;

impl Display {
    /// Create a new display instance.
    pub fn new(i2c: I2C) -> Result<Self> {
        let display_interface = I2CDisplayInterface::new(i2c);
        let mut driver = Ssd1306::new(
            display_interface,
            DisplaySize128x32,
            DisplayRotation::Rotate0,
        )
        .into_terminal_mode();

        driver.init()?;
        Ok(Display { driver })
    }

    /// Clears the display
    pub fn clear(&mut self) -> Result<()> {
        self.driver.clear()?;
        Ok(())
    }
}

impl Write for Display {
    fn write_str(&mut self, s: &str) -> core::result::Result<(), Error> {
        write!(self.driver, "{}", s)
    }
}

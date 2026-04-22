//! TCA6424A 24-bit I2C GPIO expander driver.
//!
//! Supports both synchronous and asynchronous I2C via feature flags:
//! - Default: synchronous (`embedded-hal::i2c::I2c`)
//! - `async` feature: asynchronous (`embedded-hal-async::i2c::I2c`)
//!
//! Logging is optional:
//! - `defmt` feature: structured logging for embedded/no_std
//! - `log` feature: standard `log` crate for std environments

#![no_std]

pub mod reg {
    pub const INPUT_PORT0: u8 = 0x00;
    pub const OUTPUT_PORT0: u8 = 0x04;
    pub const CONFIG_PORT0: u8 = 0x0C;
    /// Auto-increment variant of INPUT_PORT0.
    pub const AI_INPUT_PORT0: u8 = 0x80;
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    I2c(E),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Port {
    P0 = 0,
    P1 = 1,
    P2 = 2,
}

macro_rules! log_info {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        defmt::info!($($arg)*);
        #[cfg(all(feature = "log", not(feature = "defmt")))]
        log::info!($($arg)*);
    }};
}

/// TCA6424A GPIO expander with shadow registers.
///
/// All output and config registers default to `0xFF` (hardware power-on state).
pub struct Tca6424a<I2C> {
    i2c:    I2C,
    addr7:  u8,
    output: [u8; 3],
    config: [u8; 3],
}

// The I2C trait bound differs between sync and async, so we need two impl
// blocks. `maybe_async` handles the async/await stripping within each.
#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

impl<I2C, E> Tca6424a<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C, addr7: u8) -> Self {
        Self {
            i2c,
            addr7,
            output: [0xFF; 3],
            config: [0xFF; 3],
        }
    }

    /// Initialize: set port directions and push initial output values.
    #[maybe_async::maybe_async]
    pub async fn init(&mut self, config: [u8; 3], output: [u8; 3]) -> Result<(), Error<E>> {
        for port in 0..3u8 {
            self.write_reg(reg::CONFIG_PORT0 + port, 0xFF).await?;
        }
        self.config = [0xFF; 3];

        self.output = output;
        for (port, &val) in output.iter().enumerate() {
            self.write_reg(reg::OUTPUT_PORT0 + port as u8, val).await?;
        }
        for (port, &val) in config.iter().enumerate() {
            self.write_reg(reg::CONFIG_PORT0 + port as u8, val).await?;
            self.config[port] = val;
        }

        log_info!("TCA6424A @ 0x{:02X} initialized", self.addr7);
        Ok(())
    }

    /// Set output bits on a port. Masked bits are replaced with `value`.
    #[maybe_async::maybe_async]
    pub async fn set_output(
        &mut self,
        port: Port,
        value: u8,
        mask: u8,
    ) -> Result<(), Error<E>> {
        let idx = port as usize;
        let new = (self.output[idx] & !mask) | (value & mask);
        if new != self.output[idx] {
            self.write_reg(reg::OUTPUT_PORT0 + idx as u8, new).await?;
            self.output[idx] = new;
        }
        Ok(())
    }

    /// Set an entire port's output value.
    #[maybe_async::maybe_async]
    pub async fn set_port_output(&mut self, port: Port, value: u8) -> Result<(), Error<E>> {
        self.set_output(port, value, 0xFF).await
    }

    /// Set an entire port's direction config.
    #[maybe_async::maybe_async]
    pub async fn set_port_config(&mut self, port: Port, value: u8) -> Result<(), Error<E>> {
        let idx = port as usize;
        if value != self.config[idx] {
            self.write_reg(reg::CONFIG_PORT0 + idx as u8, value).await?;
            self.config[idx] = value;
        }
        Ok(())
    }

    /// Read all three input ports (auto-increment).
    #[maybe_async::maybe_async]
    pub async fn read_inputs(&mut self) -> Result<[u8; 3], Error<E>> {
        let mut buf = [0u8; 3];
        self.i2c
            .write(self.addr7, &[reg::AI_INPUT_PORT0])
            .await
            .map_err(Error::I2c)?;
        self.i2c
            .read(self.addr7, &mut buf)
            .await
            .map_err(Error::I2c)?;
        Ok(buf)
    }

    pub fn output(&self, port: Port) -> u8 {
        self.output[port as usize]
    }

    pub fn config(&self, port: Port) -> u8 {
        self.config[port as usize]
    }

    pub fn addr(&self) -> u8 {
        self.addr7
    }

    #[maybe_async::maybe_async]
    async fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), Error<E>> {
        self.i2c
            .write(self.addr7, &[reg, val])
            .await
            .map_err(Error::I2c)
    }
}

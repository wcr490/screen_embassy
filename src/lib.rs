#![no_std]
pub mod ssd1306;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<Inner: core::fmt::Debug> {
    Bus(Inner),
    Range(()),
}

pub trait ScreenCommand {
    fn raw(&self) -> u16;
}

pub struct ScreenI2c<T>
where
    T: embedded_hal_async::i2c::I2c,
{
    bus: T,
}
impl<E: embedded_hal_async::i2c::Error> From<E> for Error<E> {
    fn from(value: E) -> Self {
        Error::Bus(value)
    }
}

impl<'a, T> ScreenI2c<T>
where
    T: embedded_hal_async::i2c::I2c,
{
    pub fn new(bus: T) -> Self {
        ScreenI2c { bus }
    }
    pub async fn read_byte<C: ScreenCommand>(
        &mut self,
        address: u8,
        commond: C,
    ) -> Result<u8, Error<T::Error>> {
        self.bus
            .write(address, &commond.raw().to_be_bytes())
            .await?;
        let mut buffer = [0; 1];
        self.bus.read(address, &mut buffer).await?;
        Ok(u8::from_be_bytes(buffer))
    }
    pub async fn read_word<C: ScreenCommand>(
        &mut self,
        address: u8,
        commond: C,
    ) -> Result<u16, Error<T::Error>> {
        self.bus
            .write(address, &commond.raw().to_be_bytes())
            .await?;
        let mut buffer = [0; 2];
        self.bus.read(address, &mut buffer).await?;
        Ok(u16::from_be_bytes(buffer))
    }
    pub async fn write_byte (
        &mut self,
        address: u8,
        byte: u8,
    ) -> Result<(), Error<T::Error>> {
        self.bus.write(address, &byte.to_be_bytes()).await?;
        Ok(())
    }
    pub async fn write_command<C: ScreenCommand>(
        &mut self,
        address: u8,
        command: C,
    ) -> Result<(), Error<T::Error>> {
        self.bus.write(address, &command.raw().to_be_bytes()).await?;
        Ok(())
    }
    pub async fn write_raw_command(
        &mut self,
        address: u8,
        command: u16,
    ) -> Result<(), Error<T::Error>> {
        self.bus.write(address, &command.to_be_bytes()).await?;
        Ok(())
    }
}

#[macro_export]
macro_rules! command_raw {
    ($cmd: ident) => {
       Command::$cmd.raw()
    };
}

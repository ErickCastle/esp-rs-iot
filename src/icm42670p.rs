use embedded_hal::blocking::i2c;

#[derive(Clone, Copy)]
pub enum DeviceAddr {
    AD0 = 0b110_1000,
    AD1 = 0b110_1001,
}

#[derive(Clone, Copy)]
enum Register {
    AccelDataX1 = 0x0B, // Upper byte of Accel X-axis data
    AccelDataX0 = 0x0C, // Lower byte of Accel X-axis data
    AccelDataY1 = 0x0D, // Upper byte of Accel Y-axis data
    AccelDataY0 = 0x0E, // Lower byte of Accel Y-axis data
    AccelDataZ1 = 0x0F, // Upper byte of Accel Z-axis data
    AccelDataZ0 = 0x10, // Lower byte of Accel Z-axis data
    PwrMgmt0 = 0x1F,    // Power Management register
}

enum PwrMgmt0Bits {
    // AccelGyroOff = 0b0000_0000,
    // GyroStandByMode = 0b0000_0100,
    // GyroLowNoiseMOde = 0b0000_1100,
    // AccelLowPowerMode = 0b0000_0010,
    AccelLowNoiseMode = 0b0000_0011,
}

impl Register {
    fn address(&self) -> u8 {
        *self as u8
    }
}

pub struct ICM42670P<I2C> {
    i2c: I2C,
    address: DeviceAddr,
}

impl<I2C, E> ICM42670P<I2C>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    pub fn new(i2c: I2C, address: DeviceAddr) -> Result<Self, E> {
        Ok(Self { i2c, address })
    }

    fn write_register(&mut self, register: Register, data: u8) -> Result<(), E> {
        self.i2c
            .write(self.address as u8, &[register.address(), data])
    }

    fn read_register(&mut self, register: Register) -> Result<u8, E> {
        let mut register_value: [u8; 1] = [0];
        self.i2c.write_read(
            self.address as u8,
            &[register.address()],
            &mut register_value,
        )?;
        Ok(u8::from_le_bytes(register_value))
    }

    fn read_register_as_u16(
        &mut self,
        register_upper_byte: Register,
        register_lower_byte: Register,
    ) -> Result<u16, E> {
        let upper_byte = self.read_register(register_upper_byte)?;
        let lower_byte = self.read_register(register_lower_byte)?;

        Ok(((upper_byte as u16) << 8) | (lower_byte as u16))
    }

    pub fn set_accel_in_low_noise_mode(&mut self) -> Result<(), E> {
        self.write_register(Register::PwrMgmt0, PwrMgmt0Bits::AccelLowNoiseMode as u8)
    }

    pub fn read_accel_x(&mut self) -> Result<u16, E> {
        self.read_register_as_u16(Register::AccelDataX1, Register::AccelDataX0)
    }

    pub fn read_accel_y(&mut self) -> Result<u16, E> {
        self.read_register_as_u16(Register::AccelDataY1, Register::AccelDataY0)
    }

    pub fn read_accel_z(&mut self) -> Result<u16, E> {
        self.read_register_as_u16(Register::AccelDataZ1, Register::AccelDataZ0)
    }
}

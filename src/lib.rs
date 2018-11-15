
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

extern crate embedded_hal as hal;
use hal::blocking::i2c;

/// TLC59208 driver
pub struct Tlc59208<I2C> {
    i2c: I2C,
    address: u8,
    state: [u8; 8]
}

pub const CONTROL_REG: u8 = 0x80;
pub const AUTO_INCREMENT_REG: u8 = 0xA2;

impl<I2C, E> Tlc59208<I2C>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Creates a new driver associated with an I2C peripheral
    ///
    /// You'll likely want to setup the device after this
    pub fn new(i2c: I2C, address: u8) -> Result<Self, E> {
        let tlc59208 = Tlc59208 {
            i2c: i2c,
            address: address,
            state: [0; 8]
        };

        Ok(tlc59208)
    }

    pub fn setup(&mut self) -> Result<(), E> {
        let data = [
            CONTROL_REG,   // Write the following to control register
            0x81,   // 00h: MODE1 -> Turn on register auto increment.  Default would be 0x31
            0x03,   // 01h: MODE2 -> Default.  No WDT.
            0x00,   // 02h: PWM0
            0x00,   // 03h: PWM1
            0x00,   // 04h: PWM2
            0x00,   // 05h: PWM3
            0x00,   // 06h: PWM4
            0x00,   // 07h: PWM5
            0x00,   // 08h: PWM6
            0x00,   // 09h: PWM7
            0xFF,   // 0Ah: GRPPWM  -> Default global brightness (100%)
            0x00,   // 0Bh: GRPFREQ -> Default global blink period
            0xAA,   // 0Ch: LEDOUT0 -> Individal brightness control for all LEDs via PWM register
            0xAA,   // 0Dh: LEDOUT1 -> Individal brightness control for all LEDs via PWM register
            0x92,   // 0Eh: SUBADR1 -> Default sub-address
            0x94,   // 0Fh: SUBADR2 -> Default sub-address
            0x98,   // 10h: SUBADR3 -> Default sub-address
            0xD0    // 11h: ALLCALLADR -> Default all-call address
        ];

        self.i2c.write(self.address, &data)
    }

    pub fn set(&mut self, channel: u8, value: u8) {
        if channel < 8 {
            self.state[channel as usize] = value;
        }
    }

    // This assumes the following channel to LED hookup:
    //      group 0: 0,1,2 -> BLUE, GREEN, RED
    //      group 1: 4,5,6 -> BLUE, GREEN, RED
    pub fn rgb(&mut self, group: u8, (r, g, b): (u8, u8, u8)) {
        let mut offset = 0;

        if group == 1 {
            offset = 4;
        }

        self.state[offset+0] = b;
        self.state[offset+1] = g;
        self.state[offset+2] = r;
    }

    // This assumes that IC channel 3 and 7 are aux LEDs for RGB group 0 and 1 (respectively)
    pub fn aux(&mut self, group: u8, value: u8) {
        if group == 0 {
            self.state[3] = value;
        } else if group == 1 {
            self.state[7] = value;
        }
    }

    // Set internal state for all channels to 0 and update the I2C device
    pub fn off(&mut self) -> Result<(), E> {
        self.state = [0; 8];
        self.update()
    }

    // Send all channel states to the I2C device
    pub fn update(&mut self) -> Result<(), E> {
        let mut payload = [0; 9];

        payload[0] = AUTO_INCREMENT_REG;
        payload[1..9].copy_from_slice(&self.state);

        self.i2c.write(self.address, &payload)
    }

}
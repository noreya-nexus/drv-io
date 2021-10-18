#![allow(dead_code)]

use std::io::{Error, ErrorKind};

use super::pinconfig::PinConfig;

pub struct PowerConfig {
    device_id: u8,
    pins: Vec<PinConfig>,

    idle_3v3: u16,
    idle_5v0: u16,
    idle_12v: u16,

}

impl PowerConfig {
    pub(crate) fn new(frame: Vec<u8>) -> Result<PowerConfig, Error> {
        let mut pins: Vec<PinConfig> = vec![];
        let device_id = match frame[0..4] {
            [id, 0x03, 0x03, 0x02] | [id, 0x03, 0x03, 0x03] => id,
            _ => return Err(Error::new(ErrorKind::InvalidData, format!("Wrong frame header: {:?}", &frame[0..4]))),
        };

        let payload = &frame.as_slice()[4..];
        if payload.len() % 2 != 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Wrong length"));
        }

        for i in (0..payload.len()).step_by(3) {
            let mut tmp: u16 = 0;
            tmp |= (payload[i + 1] as u16) << 8;
            tmp |= payload[i + 2] as u16;

            pins.push(PinConfig::new(payload[i], tmp))
        }

        return Ok(PowerConfig { device_id, pins, idle_3v3: 0, idle_5v0: 0, idle_12v: 0 });
    }

    pub fn get_power_3v3(&self) -> u16 {
        return self.idle_3v3;
    }

    pub fn get_power_5v5(&self) -> u16 {
        let mut power: u16 = 0;
        for pin in &self.pins {
            if pin.voltage() == 0 {
                power += pin.current() * 5 + 200;
            } else {
                power += 400; // 400mW for each 12V config is used on 5V rail!
            }
        }
        return self.idle_5v0 + power;
    }

    pub fn get_power_12v(&self) -> u16 {
        let mut power: u16 = 0;
        for pin in &self.pins {
            if pin.voltage() == 1 {
                power += pin.current() * 12;
            }
        }

        return self.idle_12v + power;
    }


    pub(crate) fn get_device_id(&self) -> u8 {
        return self.device_id;
    }

    pub fn pin(&self, index: usize) -> Option<&PinConfig> {
        return self.pins.get(index);
    }
    pub fn pin_vec(&self) -> Vec<(u8, u16)> {
        let mut result: Vec<(u8, u16)> = vec![];
        for pin in &self.pins {
            result.push((pin.voltage(), pin.current()))
        }
        result
    }

    pub fn set_idle_power_3v3(&mut self, power: u16) {
        self.idle_3v3 = power;
    }

    pub fn set_idle_power_5v0(&mut self, power: u16) {
        self.idle_5v0 = power;
    }

    pub fn set_idle_power_12v(&mut self, power: u16) {
        self.idle_12v = power;
    }
}

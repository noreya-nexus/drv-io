#[derive(Debug)]
pub struct PinConfig {
    voltage: u8,
    current: u16,
}

impl PinConfig {
    pub fn new(voltage: u8, current: u16) -> PinConfig {
        PinConfig { voltage, current }
    }
    pub fn voltage(&self) -> u8 {
        self.voltage
    }
    pub fn current(&self) -> u16 {
        self.current
    }
}
use std::io::{Error, ErrorKind};
use std::thread::sleep;
use std::time::{Duration, Instant};

use nexus_unity_sdbp::datatypes::Descriptor;
use nexus_unity_sdbp::drv::core::SharedStats;

pub struct PowerMgmtHelper {
    slot: u16,
    dev: Descriptor,
}


impl PowerMgmtHelper {
    pub fn new(slot: u16, shared: &mut SharedStats) -> Result<PowerMgmtHelper, Error> {
        let mut stats = shared.read();
        let mut desc = None;
        for device in stats.get_devices() {
            if device.adr() == slot as u16 {
                desc = Some(device.clone());
            }
        }

        if desc.is_none() {
            return Err(Error::new(ErrorKind::NotConnected, format!("Slot {} not connected", slot)));
        }

        return Ok(PowerMgmtHelper {
            slot,
            dev: desc.unwrap(),

        });
    }

    pub fn wait_for_update_descriptor(&mut self, shared: &mut SharedStats, timeout: Duration) -> Result<(), Error> {
        let mut stats;

        let now = Instant::now();
        while now.elapsed() < timeout {
            stats = shared.read();
            for device in stats.get_devices() {
                if device.adr() == self.dev.adr() &&
                    device.uid() != self.dev.uid() {
                    debug!("{}",device);
                    trace!("Slot {} reconnect was successful",self.slot);
                    self.dev = device.clone();
                    return Ok(());
                }
            }
            sleep(Duration::from_millis(100));
        }
        return Err(Error::new(ErrorKind::TimedOut, format!("Timeout waiting for slot {} reconnect", self.slot)));
    }

    pub fn get_descriptor(&self) -> &Descriptor {
        return &self.dev;
    }
}
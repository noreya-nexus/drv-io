use std::io::{Error, ErrorKind};
use std::time::Duration;

use nexus_unity_sdbp::*;
use nexus_unity_sdbp::drv::api::{Error as ApiError, IntoBytes, Tag, TlvValue};
use nexus_unity_sdbp::drv::core::*;
use nexus_unity_sdbp::powermgmt::manager::PowerManager;
use nexus_unity_sdbp::sdbp::*;
use nexus_unity_sdbp::sdbp::response::SdbpResponse;
use nexus_unity_sdbp::util::*;
use sdbp::request::custom::io::IoBuilder;
use sdbp::response::custom::io::powermgmt::SetPowerConfig as SetPowerConfigResponse;
use sdbp::response::custom::io::powermgmt::TestPowerConfig as TestPowerConfigResponse;

use crate::powermgmt::data::PowerConfig;

use super::settings;

mod data;
mod helper;

pub struct PowerMgmt<'a, 'b> {
    vdev_id: u16,
    dev_pair: &'a ChannelPair<PMsg>,
    shared: &'b mut SharedStats,
}

impl<'a, 'b> PowerMgmt<'a, 'b> {
    pub fn new(vdev_id: u16, dev_pair: &'a ChannelPair<PMsg>, shared: &'b mut SharedStats) -> PowerMgmt<'a, 'b> {
        return PowerMgmt { vdev_id, dev_pair, shared };
    }

    fn parse(msg: &PMsg) -> Result<data::PowerConfig, Error> {
        let request = msg.get_msg().unwrap();
        let config = data::PowerConfig::new(request);

        let config = match config {
            Err(err) => return Err(err),
            Ok(value) => value,
        };
        return Ok(config);
    }


    fn suspend_device(&mut self, dev_id: u16) -> Result<(), Error> {
        let cmd_mode_suspend = CoreBuilder::new().control().mode_suspend().unwrap();

        let dev_msg = PMsg::create(self.vdev_id, dev_id as u16, Ok(cmd_mode_suspend));
        match self.dev_pair.tx().send(dev_msg) {
            Ok(_) => (),
            Err(err) => {
                error!("{}",err);
                return Err(Error::new(ErrorKind::BrokenPipe, format!("Sending command to slot {} failed", dev_id)));
            }
        };

        match self.dev_pair.rx().recv_timeout(Duration::from_millis(1000)) {
            Ok(_) => (),
            Err(err) => {
                error!("{}",err);
                return Err(Error::new(ErrorKind::BrokenPipe, format!("Receiving from slot {} failed", dev_id)));
            }
        };
        return Ok(());
    }

    fn update_descriptor(&mut self, dev_id: u16) -> Result<(), Error> {
        let mut helper = match helper::PowerMgmtHelper::new(dev_id, &mut self.shared) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };

        let request = sdbp::request::core::control::ControlBuilder::new().update_descriptor().unwrap();
        let device_msg = PMsg::create(self.vdev_id, dev_id as u16, Ok(request));
        match self.dev_pair.tx().send(device_msg) {
            Ok(_) => (),
            Err(err) => {
                error!("{}",err);
                return Err(Error::new(ErrorKind::BrokenPipe, format!("Sending command to slot {} failed", dev_id)));
            }
        }

        match self.dev_pair.rx().recv_timeout(Duration::from_millis(1000)) {
            Ok(_) => (),
            Err(err) => {
                error!("{}",err);
                return Err(Error::new(ErrorKind::BrokenPipe, format!("Receiving from slot {} failed", dev_id)));
            }
        }
        return helper.wait_for_update_descriptor(&mut self.shared, Duration::from_millis(600));
    }

    fn update_config(&mut self, conifg: &mut PowerConfig) -> Result<(), Error> {
        let helper = match helper::PowerMgmtHelper::new(conifg.get_device_id() as u16, &mut self.shared) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };

        let device = helper.get_descriptor();

        conifg.set_idle_power_3v3(device.max_power_3v3());
        //conifg.set_idle_power_5v0(device.max_power_5v());
        //conifg.set_idle_power_12v(device.max_power_12v());

        Ok(())
    }


    fn test_power_config(&self, config: &PowerConfig) -> Result<(), Error> {
        for entry in config.pin_vec() {
            info!("{:?}",entry);
        }
        trace!("Slot {}: test power config",config.get_device_id());
        let cmd_test_pwr_config = match IoBuilder::new().powermgmt().test_power_config(config.pin_vec()) {
            Ok(value) => value,
            Err(err) => {
                error!("{}", err);
                return Err(err);
            }
        };
        info!("cmd_test_pwr_config: {:?}", &cmd_test_pwr_config);

        let device_msg = PMsg::create(self.vdev_id, config.get_device_id() as u16, Ok(cmd_test_pwr_config));
        match self.dev_pair.tx().send(device_msg) {
            Ok(_) => (),
            Err(err) => {
                error!("{}",err);
                return Err(Error::new(ErrorKind::BrokenPipe, format!("Sending command to slot {} failed", config.get_device_id())));
            }
        }

        let response = match self.dev_pair.rx().recv_timeout(Duration::from_millis(1000)) {
            Ok(value) => value,
            Err(err) => {
                error!("{}",err);
                return Err(Error::new(ErrorKind::BrokenPipe, format!("Receiving from slot {} failed", config.get_device_id())));
            }
        };

        let pm_test_response = match TestPowerConfigResponse::from_raw(response.get_msg().unwrap()) {
            Ok(value) => value,
            Err(err) => {
                error!("{}", err);
                return Err(Error::new(ErrorKind::BrokenPipe, format!("Receiving from slot {} failed", config.get_device_id())));
            }
        };

        if pm_test_response.status != 0 {
            error!("Error in response test_power_config from slot {}",config.get_device_id());
            return Err(Error::new(ErrorKind::InvalidInput, format!("Invalid power config")));
        }
        Ok(())
    }

    fn set_power_config(&self, config: &PowerConfig) -> Result<(), Error> {
        trace!("Slot {}: set power config",config.get_device_id());
        let cmd_set_pwr_config = match IoBuilder::new().powermgmt().set_power_config(config.pin_vec()) {
            Ok(value) => value,
            Err(err) => {
                error!("{}", err);
                return Err(err);
            }
        };
        info!("cmd_set_pwr_config: {:?}", &cmd_set_pwr_config);

        let device_msg = PMsg::create(self.vdev_id, config.get_device_id() as u16, Ok(cmd_set_pwr_config));

        match self.dev_pair.tx().send(device_msg) {
            Ok(_) => (),
            Err(err) => {
                error!("{}",err);
                return Err(Error::new(ErrorKind::BrokenPipe, format!("Sending command to slot {} failed", config.get_device_id())));
            }
        }

        let response = match self.dev_pair.rx().recv_timeout(Duration::from_millis(1000)) {
            Ok(value) => value,
            Err(err) => {
                error!("{}",err);
                return Err(Error::new(ErrorKind::BrokenPipe, format!("Receiving from slot {} failed", config.get_device_id())));
            }
        };

        let pm_set_response = match SetPowerConfigResponse::from_raw(response.get_msg().unwrap()) {
            Ok(value) => value,
            Err(err) => {
                error!("{}", err);
                return Err(Error::new(ErrorKind::BrokenPipe, format!("Parsing Response (SetPowerConfigResponse) from slot {} failed", config.get_device_id())));
            }
        };

        if pm_set_response.status != 0 {
            error!("Error in response set_power_config from slot {}",config.get_device_id());
            error!("CODE: {}",pm_set_response.status);
            return Err(Error::new(ErrorKind::InvalidInput, format!("Invalid power config")));
        }
        Ok(())
    }

    // fn is_update_necessary(&mut self, conf: &PowerConfig) -> Result<bool,Error> {
    //
    //     let mut helper = match helper::PowerMgmtHelper::new(conf.get_device_id() as u16, &mut self.shared) {
    //         Ok(value) => value,
    //         Err(err) => return Err(err),
    //     };
    //
    //     let desc = helper.get_descriptor();
    //
    //     let mut result = true;
    //
    //     if  desc.max_power_5v() == conf.get_power_5v5() &&
    //         desc.max_power_12v() == conf.get_power_12v() {
    //         result = false;
    //     }
    //     return Ok(result)
    //
    // }

    fn power_mangement(&mut self, msg: &PMsg) -> Result<(), Error> {
        let mut cmd = match PowerMgmt::parse(&msg) {
            Ok(value) => value,
            Err(err) => {
                error!("{}",err);
                return Err(err);
            }
        };

        // Note: If the rest driver sends a suspend this breaks the power mgmt
        // match self.is_update_necessary(&cmd) {
        //     Ok(value) => {
        //         if value == false {
        //             return Ok(());
        //         }
        //     },
        //     Err(err) => return Err(err),
        // }

        //let _ = self.reset_device(cmd.get_device_id() as u16);
        let _ = self.suspend_device(cmd.get_device_id() as u16);
        let _ = self.update_descriptor(cmd.get_device_id() as u16);

        match self.update_config(&mut cmd) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }

        debug!("test_power_config");
        match self.test_power_config(&cmd) {
            Ok(_) => (),
            Err(err) => {
                error!("{}",err);
                return Err(err);
            }
        }

        let mut con_pm = match PowerManager::new(settings::POWER_MGMT_PATH.to_string(), Some(Duration::from_secs(1))) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        debug!("3v3: {:?} 5v0: {:?} 12v: {:?}",cmd.get_power_3v3(),cmd.get_power_5v5(),cmd.get_power_12v());
        let response = con_pm.request(cmd.get_device_id(), cmd.get_power_3v3(), cmd.get_power_5v5(), cmd.get_power_12v());
        match response.successful {
            true => (),
            false => return Err(Error::new(ErrorKind::InvalidInput, format!("invalid power config requested too much power (3v3: {}, 5v0: {},12v: {}", response.to_much_power_3v3, response.to_much_power_5v0, response.to_much_power_12v)))
        }

        debug!("set_power_config");
        match self.set_power_config(&cmd) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }

        debug!("update_descriptor");
        match self.update_descriptor(cmd.get_device_id() as u16) {
            Ok(_) => (),

            Err(err) => {
                return Err(err);
            }
        }

        debug!("finish request");
        let response = con_pm.finish_request();
        match response.successful {
            true => (),
            false => {
                error!("FINISH ERROR");
                return Err(Error::new(ErrorKind::InvalidInput, format!("finish power config failed")));
            }
        }
        return Ok(());
    }

    pub fn execute(&mut self, msg: &PMsg) -> PMsg {
        let mut tlv = TlvValue::new();
        tlv[Tag::DeviceTunnel] = TlvValue::new_array();

        let response_ok: Vec<u8> = vec![0x00];

        match self.power_mangement(msg) {
            Ok(_) => {
                tlv[Tag::DeviceTunnel] = TlvValue::new_array();
                tlv[Tag::DeviceTunnel][Tag::Response] = TlvValue::Bytes(response_ok);
            }
            Err(err) => {
                error!("{}",err);
                tlv[Tag::DeviceTunnel][Tag::ErrorValue] = TlvValue::U16(ApiError::VirtualDeviceError as u16);
                tlv[Tag::DeviceTunnel][Tag::ErrorMsg] = TlvValue::String(format!("{}", err));
            }
        };

        PMsg::create(msg.get_dst(), msg.get_src(), Ok(tlv.into_bytes()))
    }

    pub fn handle_function(vdev_id: u16, ctl_pair: ChannelPair<ManagedThreadState>, dev_pair: ChannelPair<PMsg>, mut shared: SharedStats) {
        let mut stopped = false;

        debug!("Started {} ", std::thread::current().name().unwrap());

        let mut mgmt = PowerMgmt::new(vdev_id, &dev_pair, &mut shared);

        while !stopped {
            ManagedThreadUtil::is_stopped(&mut stopped, &ctl_pair);

            let res = match dev_pair.rx().recv_timeout(Duration::from_millis(150)) {
                Ok(value) => {
                    mgmt.execute(&value)
                }
                Err(_err) => continue,
            };

            match dev_pair.tx().send(res) {
                Err(_) => error!("Error while sending response for to client"),
                _ => (),
            }
        }
        debug!("Stopped {}", std::thread::current().name().unwrap());
    }
}
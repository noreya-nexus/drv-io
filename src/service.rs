use std::time::{Duration};

use nexus_unity_sdbp::datatypes::Descriptor;
use nexus_unity_sdbp::util::*;
use nexus_unity_sdbp::drv::core::*;
use nexus_unity_sdbp::sdbp::*;

pub struct IoModule {}

use super::notification_handler::*;


const NO_NOTIFICATION_PENDING: [u8; 4] = [request::core::protocol::CLASS_ID, request::core::protocol::classes::notification::ID, request::core::protocol::classes::notification::operation_code::ERROR, 0x03];

impl IoModule {
    fn transfer(dev_handle: &mut DeviceHandle, buf: Vec<u8>) -> Result<Vec<u8>, std::io::Error> {
        let res = dev_handle.write(buf);
        if res.is_err() {
            return Err(res.unwrap_err());
        }

        let mut response = vec![0; 4096];
        let res = dev_handle.read(&mut response);
        match res {
            Ok(value) => return Ok(Vec::from(&response[0..value])),
            Err(_err) => return Err(_err),
        };
    }

    fn is_not_get_notification(raw: &[u8]) -> bool {
        if raw[0] == request::core::protocol::CLASS_ID &&
            raw[1] == request::core::protocol::classes::notification::ID &&
            raw[2] == request::core::protocol::classes::notification::operation_code::GET_NOTIFICATION {
            return false;
        }
        return true;
    }

    fn is_suspend(raw: &[u8]) -> bool {
        if raw[0] == request::core::protocol::CLASS_ID &&
            raw[1] == request::core::protocol::classes::control::ID &&
            raw[2] == request::core::protocol::classes::control::operation_code::MODE_SUSPEND {
            return true;
        }
        return false;
    }

    pub fn handle_function(desc: Descriptor, ctl_pair: ChannelPair<ManagedThreadState>, dev_pair: ChannelPair<PMsg>) {
        let mut stopped = false;
        let mut err_cnt: u32 = 0;
        debug!("Started {} for {}" ,std::thread::current().name().unwrap(),desc.path().to_str().unwrap());


        let tmp = desc.clone();
        let (notification_chn, notification_sender) = ChannelPair::new();
        let notification_handler = spawn("NotificationHandler".to_string(), |inner_ctl_pair| NotificationHandler::task(tmp, inner_ctl_pair, notification_sender));


        let mut latest_notification: Option<Vec<u8>> = None;

        while !stopped {
            ManagedThreadUtil::is_stopped(&mut stopped, &ctl_pair);


            //Init Sequence
            let result = DeviceHandle::new(&desc.dev_file());
            let mut dev_handle = match result {
                None => {
                    trace!("{:?} - Cannot open device file", desc.dev_file());
                    continue;
                }
                Some(value) => value,
            };

            info!("Setting communication speed to: {} kHz",desc.max_sclk_speed());
            match IoModule::transfer(&mut dev_handle, CoreBuilder::new().control().set_sclk_speed(desc.max_sclk_speed()).unwrap()) {
                Ok(response) => {
                    if response[0] != 0x01 || response[1] != 0x03 || response[2] != 0x08 || response[3] != 0x00 {
                        panic!("Communication speed change failed")
                    }
                }
                Err(_) => { panic!("Failed setting communication speed") }
            };


            while !stopped {
                ManagedThreadUtil::is_stopped(&mut stopped, &ctl_pair);

                let com_result = &dev_pair.rx().recv_timeout(Duration::from_millis(50));
                let mut reset_notification = false;
                if com_result.is_ok() {
                    let msg = com_result.as_ref().unwrap();
                    trace!("{:?} - rx - {:?}",&desc.path().to_str().unwrap(),msg);

                    //if(msg.get_msg() == sdbp::CoreBuilder::new())

                    let command = msg.get_msg().unwrap();

                    if IoModule::is_suspend(command.as_slice()) {
                        reset_notification = true;
                    }

                    if IoModule::is_not_get_notification(command.as_slice()) {
                        let mut response = Err(std::io::Error::from(std::io::ErrorKind::NotConnected));
                        for i in 0..3 {
                            let ret = IoModule::transfer(&mut dev_handle, msg.get_msg().unwrap());

                            if ret.is_err() {
                                if i == 2 {
                                    response = ret;
                                }
                                err_cnt += 1;
                            } else {
                                response = ret;
                                break;
                            }
                        }
                        let answer = PMsg::create(msg.get_dst(), msg.get_src(), response);
                        trace!("{:?} - tx - {:?}",&desc.path().to_str().unwrap(),msg);
                        let _ = dev_pair.tx().send(answer);
                    } else {
                        let answer = match &latest_notification {
                            None => PMsg::create(msg.get_dst(), msg.get_src(), Ok(Vec::from(NO_NOTIFICATION_PENDING))),
                            Some(value) => {
                                PMsg::create(msg.get_dst(), msg.get_src(), Ok(value.clone()))
                            }
                        };
                        trace!("{:?} - tx - {:?}", &desc.path().to_str().unwrap(), msg);
                        let _ = dev_pair.tx().send(answer);
                        latest_notification = None;
                    }
                }

                let result = notification_chn.rx().recv_timeout(Duration::from_millis(50));
                match result {
                    Ok(value) => {
                        latest_notification = Some(value.get_msg().unwrap());
                        debug!("Received Notification {:?}",value.get_msg().unwrap());
                    }
                    Err(_) => (),
                }
                if reset_notification {
                    latest_notification = None;
                }

                match IoModule::transfer(&mut dev_handle, FrameBuilder::new().core().control().mode_run().unwrap()) {
                    Err(_err) => {
                        err_cnt += 1;
                        trace!("Error cnt: {}", err_cnt);
                        break;
                    }
                    Ok(value) => value,
                };
            }
            drop(dev_handle);
        }
        match notification_handler.stop(Duration::from_millis(50)) {
            Ok(_) => {}
            Err(err) => {
                error!("Could not stop notification handler: {}", err.to_string())
            }
        };
        debug!("Stopped {}",std::thread::current().name().unwrap());
    }
}
use nexus_unity_sdbp::datatypes::Descriptor;
use nexus_unity_sdbp::util::*;
use nexus_unity_sdbp::drv::core::*;

use std::fs::File;
use std::io::Read;

pub struct NotificationHandler {}

impl NotificationHandler {
    pub fn task(desc: Descriptor, ctl_pair: ChannelPair<ManagedThreadState>, dev_pair: ChannelPair<PMsg>) {
        let mut stopped = false;

        info!("Start NotificationHandler");

        let path: String = format!("/sys/class/sdbp/slot{}/notification", desc.adr());

        while !stopped {
            ManagedThreadUtil::is_stopped(&mut stopped, &ctl_pair);

            let f = File::open(&path);
            let mut fh = match f {
                Ok(value) => value,
                Err(_) => {
                    error!("Cannot open notification file");
                    break;
                }
            };

            let mut buffer = Vec::new();
            let result = fh.read_to_end(&mut buffer);
            if result.is_ok() {
                let notification_string = String::from_utf8_lossy(&buffer);
                //info!("Decoded notification: {:?}", notification_string);
                //info!("Decoded notification hex: {:x?}", notification_string);
                //info!("Decoded notification length: {:?}", notification_string.len());
                let notification_string = notification_string.strip_prefix("0x").expect("Decoding notification failed");
                let notification_string = notification_string.strip_suffix("\0").expect("Decoding notification failed");
                let notification_string = hex::decode(notification_string).expect("Decoding notification failed");
                debug!("Decoded hex notification: {:x?}", notification_string);
                let msg = PMsg::create(0, 0, Ok(notification_string));
                let _ = dev_pair.tx().send(msg);
            }
            drop(fh);
        }
        info!("Stopped NotificationHandler")
    }
}
#[macro_use]
extern crate log;

use std::{env, panic, process};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use noreya_sdbp::util::logging::init_systemd_logger;
use noreya_sdbp::datatypes::*;
use noreya_sdbp::drv::core::{Controller, DeviceFilter, DeviceHandler, Dispatcher, DrvMeta, SdbpkCheck, SharedStats, Stats, UdsServer};
use noreya_sdbp::drv::service::service::SdbpModule;
use sd_notify::NotifyState;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;

use powermgmt::PowerMgmt;

use crate::settings::{COMPATIBLE_FW_MAJOR, COMPATIBLE_FW_MINOR};

pub mod settings;
pub mod powermgmt;

fn main() {
    init_systemd_logger();
    let version = env!("CARGO_PKG_VERSION");
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // This ends the entire process if one thread panics
        orig_hook(panic_info);
        process::exit(1);
    }));

    let check = SdbpkCheck {
        major: 1,
        minor: 3,
        patch: 0
    };

    let check = match check.check_version() {
        Ok(version) => {
            info!("SDBPK driver version: {}.{}.{}", version.major, version.minor, version.patch);
            version
        }
        Err(err) => {
            error!("{}",err);
            exit(-1)
        }
    };

    info!("Module driver version: {}",version);

    let mut signals = Signals::new(&[SIGTERM,SIGINT]).ok().unwrap();
    let mut filter  = DeviceFilter::<String>::new();
    filter.add(settings::MODULE_NAME.to_string());

    /*
     * Prepare Global Settings
     */
    let shared = SharedStats::new(Stats::new(settings::MODULE_NAME.to_string(),Version::from_str(version).unwrap(),check.to_version()));

    /*
     * Device-Event channels
     */
    let (devt_sender,devt_receiver) = crossbeam_channel::unbounded();

    let device_handler = DeviceHandler::start(filter,devt_receiver.clone(),devt_sender.clone());
    let dispatcher = Dispatcher::start();
    let controller = Controller::start(dispatcher.get_com(), devt_receiver.clone(), shared.clone(), SdbpModule::handle_function, COMPATIBLE_FW_MAJOR, COMPATIBLE_FW_MINOR);

    let meta = DrvMeta::new(settings::MODULE_NAME.to_string(),settings::DRV_NAME.to_string(),settings::SOCKET_PATH.to_string());
    let udsserver = UdsServer::start(meta,dispatcher.get_com(),shared.clone());

    let power_mgmt = Controller::start_virtual_device("PowerMgmt".to_string(), 0x2001, &mut dispatcher.get_com(), shared.clone(), PowerMgmt::handle_function);

    info!("Started driver for {}",settings::MODULE_NAME);
    let _ = sd_notify::notify(false, &[NotifyState::Ready]);
    let _ = sd_notify::notify(false, &[NotifyState::Status("Waiting for requests...")]);

    for _sig in signals.forever() {
        udsserver.stop(Duration::from_millis(100)); // Note: duration must be low for udsserver
        device_handler.stop(Duration::from_millis(1000));
        controller.stop(Duration::from_millis(1000));
        dispatcher.stop(Duration::from_millis(1000));
        power_mgmt.stop(Duration::from_millis(1000));
        break;
    }

    let _ = sd_notify::notify(false, &[NotifyState::Stopping]);
    sleep(Duration::from_secs(3)); // Wait some time to let all the threads stop...
    let _ = sd_notify::notify(false, &[NotifyState::Status("Service stopped successfully")]);
    info!("Driver service stopped")
}

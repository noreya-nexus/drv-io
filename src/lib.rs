use std::time::Duration;

pub mod settings;

use noreya_sdbp::datatypes::*;
use std::thread::sleep;
use noreya_sdbp::drv::core::{DeviceFilter, SharedStats, Stats, DeviceHandler, Dispatcher, Controller, DrvMeta, UdsServer};
use noreya_sdbp::drv::service::service::SdbpModule;
use noreya_sdbp::util::{ManagedThreadHandle, spawn, ChannelPair, ManagedThreadState};
use crate::settings::{COMPATIBLE_FW_MAJOR, COMPATIBLE_FW_MINOR};

pub fn start() -> ManagedThreadHandle<()> {

    //env::set_var("RUST_APP_LOG", "debug");
    //env::set_var("RUST_BACKTRACE", "1");

    //pretty_env_logger::init_custom_env("RUST_APP_LOG");

    spawn("drv-io main".to_string(),move |ctl_chn | start_driver(ctl_chn) )
}


pub fn start_driver(ctl_chn: ChannelPair<ManagedThreadState>) {

    let mut filter = DeviceFilter::<String>::new();
    filter.add(settings::MODULE_NAME.to_string());

    /*
     * Prepare Global Settings
     */
    let shared = SharedStats::new(Stats::new(settings::MODULE_NAME.to_string(), Version::from_str("00000.00001.00000").unwrap(),Version::from_str("00000.00001.00000").unwrap()));

    /*
     * Device-Event channels
     */
    let (devt_sender, devt_receiver) = crossbeam_channel::unbounded();

    let device_handler = DeviceHandler::start(filter, devt_receiver.clone(), devt_sender.clone());
    let dispatcher = Dispatcher::start();
    let controller = Controller::start(dispatcher.get_com(), devt_receiver.clone(), shared.clone(), SdbpModule::handle_function, COMPATIBLE_FW_MAJOR, COMPATIBLE_FW_MINOR);

    let meta = DrvMeta::new(settings::MODULE_NAME.to_string(), settings::DRV_NAME.to_string(), settings::SOCKET_PATH.to_string());
    let udsserver = UdsServer::start(meta, dispatcher.get_com(), shared.clone());

    //Wait unitl stop
    ctl_chn.rx().recv().unwrap();

    controller.stop(Duration::from_millis(1000));
    dispatcher.stop(Duration::from_millis(1000));
    device_handler.stop(Duration::from_millis(1000));
    udsserver.stop(Duration::from_millis(10000));

    sleep(Duration::from_secs(1));

    let _ = ctl_chn.tx().send(ManagedThreadState::OK);
}


pub fn simple_main() {

}
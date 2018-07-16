#[macro_use]
extern crate log;
extern crate env_logger;
extern crate dbus;

mod dbus_bluez;
mod bt_sensor;
mod config;

use std::{thread, time};

fn run<'a>() -> Result<(), Box<std::error::Error>> {
    let conf = config::SensorConf::new();
    let mut dbus = dbus_bluez::DbusBluez::new(conf)?;
    let duration = time::Duration::from_millis(500);
    dbus.initialize()?;
    loop {
        let sensors = dbus.get_sensors()?;
        for (_, sensor) in sensors {
            println!("Sensor: {}, measurements {:?}", sensor.get_address(), sensor.get_measurements())
        }
        thread::sleep(duration);
    }
}

fn main() {
    env_logger::init();
    ::std::process::exit(match run() {
        Ok(_) => 0,
        Err(e) => {
            error!("{:?}", e);
            1
        },
    });
}

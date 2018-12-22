#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
extern crate env_logger;
extern crate dbus;
extern crate serde_json;
extern crate docopt;
extern crate base64;
extern crate influx_db_client;

mod bt_sensor_factory;
mod ruuvitag_df3;
mod ruuvitag_df2;
mod dbus_bluez;
mod bt_device;
mod bt_sensor;
mod consumer;
mod config;
mod error;

use std::{thread, time};

use docopt::Docopt;

const USAGE: &'static str = "
Bluetooth Sensor Collector.

Usage:
  bt-sensor (-h | --help)
  bt-sensor --version
  bt-sensor [options]
  bt-sensor [options] <device>...

Options:
  -h --help                  Show this screen.
  --version                  Show version.
  --devicemap=<conf>         Device address to device type map file.
  --btdevice=<device>        Bluetooth device name [default: hci0].
  --auto=<mode>              Discovery mode [default: true].
  --interval=<secs>          BT device Poll interval [default: 3].
  --consumer=<type>          The consumer type [default: stdout].
  <device>                   Device address map (MAC,tag,type)
";

#[derive(Debug, Deserialize, Serialize)]
pub struct Args {
    flag_devicemap: Option<String>,
    flag_btdevice: String,
    flag_auto: bool,
    flag_interval: u64,
    flag_consumer: consumer::ConsumerType,
    arg_device: Vec<String>,
}

fn run<'a>() -> Result<(), Box<std::error::Error>> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|docopt| {
            docopt
                .version(Some("0.0.1".to_string()))
                .deserialize()
        })
        .unwrap_or_else(|e| e.exit());
    let conf = config::SensorConf::new(&args);
    let mut dbus = dbus_bluez::DbusBluez::new(conf, args.flag_btdevice.to_string())?;
    let duration = time::Duration::from_secs(args.flag_interval);
    dbus.initialize()?;
    let consumer = consumer::initialize_consumer(&args.flag_consumer)?;
    loop {
        let sensors = dbus.get_sensors()?;
        for (_, sensor) in sensors {
            match sensor.get_measurements() {
                Some(measurement) => consumer.consume(&measurement),
                None => {},
            }
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

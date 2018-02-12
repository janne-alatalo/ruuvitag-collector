#[macro_use]
extern crate log;
extern crate env_logger;
extern crate dbus;

use dbus::{Props, BusType, Connection, MessageItem};
use std::io::{Error, ErrorKind};

fn run<'a>() -> Result<(), Box<std::error::Error>> {
    let c = Connection::get_private(BusType::System)?;
    let props = Props::new(&c, "org.bluez", "/org/bluez/hci0", "org.bluez.Adapter1", 500);
    let mut is_powered = match props.get("Powered")? {
        MessageItem::Bool(b) => { b },
        _ => { panic!("Not the type that was expected!") },
    };
    if !is_powered {
        info!("Turning the bluetooth interface on...");
        props.set("Powered", dbus::MessageItem::Bool(true))?;
        is_powered = match props.get("Powered")? {
            MessageItem::Bool(b) => { b },
            _ => { panic!("Not the type that was expected!") },
        };
        if !is_powered {
            return Err(Box::new(Error::new(ErrorKind::Other, "The bluetooth interface is not powered on!")))
        } else {
            info!("Interface on");
        }
    }
    print!("{}", is_powered);
    Ok(())
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

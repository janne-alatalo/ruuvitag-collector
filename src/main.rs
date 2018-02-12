#[macro_use]
extern crate log;
extern crate env_logger;
extern crate dbus;

use dbus::{Props, BusType, Connection, MessageItem, Message};
use std::io::{Error, ErrorKind};

static BLUEZ_SERVICE: &'static str = "org.bluez";
static BLUEZ_INTERFACE_ADAPTER1: &'static str = "org.bluez.Adapter1";
static BLUEZ_OBJECT_PATH: &'static str = "/org/bluez/hci0";
static BLUEZ_START_DISCOVERY: &'static str = "StartDiscovery";

fn run<'a>() -> Result<(), Box<std::error::Error>> {
    let c = Connection::get_private(BusType::System)?;
    let props = Props::new(&c, BLUEZ_SERVICE, BLUEZ_OBJECT_PATH, BLUEZ_INTERFACE_ADAPTER1, 500);
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
            return Err(Box::new(Error::new(
                        ErrorKind::Other,"The bluetooth interface is not powered on!")))
        } else {
            info!("Interface on");
        }
    }
    let msg = Message::new_method_call(
        BLUEZ_SERVICE, BLUEZ_OBJECT_PATH, BLUEZ_INTERFACE_ADAPTER1, BLUEZ_START_DISCOVERY)?;
    c.send_with_reply_and_block(msg, 1000)?;
    let is_discovering = match props.get("Discovering")? {
        MessageItem::Bool(b) => { b },
        _ => { panic!("Not the type that was expected!") },
    };
    if !is_discovering {
        return Err(Box::new(Error::new(
                    ErrorKind::Other, "Can't set bluetooth to discover mode")))
    }
    info!("Bluetooth discovering!");
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

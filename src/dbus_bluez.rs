
use std::io;
use std::error;
use dbus;

static BLUEZ_SERVICE: &'static str = "org.bluez";
static BLUEZ_INTERFACE_ADAPTER1: &'static str = "org.bluez.Adapter1";
static BLUEZ_OBJECT_PATH: &'static str = "/org/bluez/hci0";
static BLUEZ_START_DISCOVERY: &'static str = "StartDiscovery";

pub struct DbusBluez {
    conn: dbus::Connection,
}

impl DbusBluez {

    pub fn new() -> Result<DbusBluez, Box<error::Error>> {
        let bus = DbusBluez{
            conn: dbus::Connection::get_private(dbus::BusType::System)?,
        };
        Ok(bus)
    }

    pub fn initialize(&self) -> Result<(), Box<error::Error>> {

        let props = dbus::Props::new(&self.conn, BLUEZ_SERVICE, BLUEZ_OBJECT_PATH, BLUEZ_INTERFACE_ADAPTER1, 500);
        let mut is_powered = match props.get("Powered")? {
            dbus::MessageItem::Bool(b) => { b },
            _ => { panic!("Not the type that was expected!") },
        };
        if !is_powered {
            info!("Turning the bluetooth interface on...");
            props.set("Powered", dbus::MessageItem::Bool(true))?;
            is_powered = match props.get("Powered")? {
                dbus::MessageItem::Bool(b) => { b },
                _ => { panic!("Not the type that was expected!") },
            };
            if !is_powered {
                return Err(Box::new(io::Error::new(
                            io::ErrorKind::Other,"The bluetooth interface is not powered on!")))
            } else {
                info!("Interface on");
            }
        }
        let msg = dbus::Message::new_method_call(
            BLUEZ_SERVICE, BLUEZ_OBJECT_PATH, BLUEZ_INTERFACE_ADAPTER1, BLUEZ_START_DISCOVERY)?;
        self.conn.send_with_reply_and_block(msg, 1000)?;
        let is_discovering = match props.get("Discovering")? {
            dbus::MessageItem::Bool(b) => { b },
            _ => { panic!("Not the type that was expected!") },
        };
        if !is_discovering {
            return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other, "Can't set bluetooth to discover mode")))
        }
        info!("Bluetooth discovering!");
        Ok(())

    }

}

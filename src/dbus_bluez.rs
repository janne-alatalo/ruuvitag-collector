use std::io;
use std::error;
use dbus;
use device;

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

    pub fn get_managed_devices(&self) -> Result<Vec<String>, Box<error::Error>> {
        let msg = dbus::Message::new_method_call(BLUEZ_SERVICE, "/",
                                             "org.freedesktop.DBus.ObjectManager",
                                             "GetManagedObjects")?;
        // Similar implementation as here:
        // https://github.com/szeged/blurz/blob/7729c462439fb692f12e385a84ab371423eb4cd6/src/bluetooth_utils.rs#L53
        let result = self.conn.send_with_reply_and_block(msg, 3000)?;
        let result_vec = result.get_items();
        let items: &[dbus::MessageItem] = result_vec.get(0).unwrap().inner().unwrap();
        for i in items {
            let (path, ifs) = i.inner().unwrap();
            let interfaces: &[dbus::MessageItem] = ifs.inner().unwrap();
            for intf in interfaces {
                let (intf_tmp, _) = intf.inner().unwrap();
                let intf_str: &str = intf_tmp.inner().unwrap();
                if intf_str == "org.bluez.Device1" {
                    let path_str: &str = path.inner().unwrap();
                    println!("{:?}", path_str);
                }
            }
        }
        Ok(Vec::new())
    }

    pub fn get_devices(&self) -> Result<Vec<device::Device>, Box<error::Error>> {
        let managed = self.get_managed_devices();
        Err(Box::new(io::Error::new(io::ErrorKind::Other, "Test")))
    }

}

use std::io;
use std::error;
use std::{thread, time};
use dbus::{
    Message, MessageItem, MessageItemArray,
    Signature, Props, Connection, BusType,
};
use device;

static BLUEZ_SERVICE: &'static str = "org.bluez";
static BLUEZ_INTERFACE_ADAPTER1: &'static str = "org.bluez.Adapter1";
static BLUEZ_INTERFACE_DEVICE1: &'static str = "org.bluez.Device1";
static BLUEZ_OBJECT_PATH: &'static str = "/org/bluez/hci0";
static BLUEZ_START_DISCOVERY: &'static str = "StartDiscovery";
static BLUEZ_SET_DISCOVERY_FILTER: &'static str = "SetDiscoveryFilter";

pub struct DbusBluez {
    conn: Connection,
}

impl DbusBluez {

    pub fn new() -> Result<DbusBluez, Box<error::Error>> {
        let bus = DbusBluez{
            conn: Connection::get_private(BusType::System)?,
        };
        Ok(bus)
    }

    pub fn initialize(&self) -> Result<(), Box<error::Error>> {

        let props = Props::new(
            &self.conn, BLUEZ_SERVICE, BLUEZ_OBJECT_PATH, BLUEZ_INTERFACE_ADAPTER1, 500);
        self.poweron_interface(&props)?;
        self.set_discovery_filter()?;
        self.start_discovering(&props)?;

        info!("Bluetooth discovering!");
        Ok(())

    }

    fn poweron_interface(&self, props: &Props) -> Result<(), Box<error::Error>> {

        let mut is_powered = match props.get("Powered")? {
            MessageItem::Bool(b) => { b },
            _ => { panic!("Not the type that was expected!") },
        };
        if !is_powered {
            info!("Turning the bluetooth interface on...");
            props.set("Powered", MessageItem::Bool(true))?;
            is_powered = match props.get("Powered")? {
                MessageItem::Bool(b) => { b },
                _ => { panic!("Not the type that was expected!") },
            };
            if !is_powered {
                return Err(Box::new(io::Error::new(
                            io::ErrorKind::Other, "The bluetooth interface is not powered on!")))
            } else {
                info!("Interface on");
            }
        }

        Ok(())

    }

    fn set_discovery_filter(&self) -> Result<(), Box<error::Error>> {

        let empty: Vec<MessageItem> = Vec::new();
        let str_arr_sign = Signature::new("a(s)")?;
        let uuid_arr = match MessageItemArray::new(empty, str_arr_sign) {
            Ok(a) => a,
            Err(e) => match e {
                _ => return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other, "ArrayError"))),
            },
        };
        let uuid_entry = MessageItem::DictEntry(
            Box::new(MessageItem::Str(String::from("UUIDs"))),
            Box::new(MessageItem::Variant(Box::new(MessageItem::Array(uuid_arr)))),
            );

        let transport_entry = MessageItem::DictEntry(
            Box::new(MessageItem::Str(String::from("Transport"))),
            Box::new(MessageItem::Variant(Box::new(MessageItem::Str(String::from("le"))))),
            );

        let dict_sign = Signature::new("a{sv}")?;
        let dict_arr = match MessageItemArray::new(vec!(uuid_entry, transport_entry), dict_sign) {
            Ok(a) => a,
            Err(e) => match e {
                _ => return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other, "ArrayError"))),
            },
        };

        let param = MessageItem::Array(dict_arr);

        let msg1 = Message::new_method_call(
            BLUEZ_SERVICE, BLUEZ_OBJECT_PATH,
            BLUEZ_INTERFACE_ADAPTER1, BLUEZ_SET_DISCOVERY_FILTER)?.append1(param);
        self.conn.send_with_reply_and_block(msg1, 1000)?;

        Ok(())

    }

    fn start_discovering(&self, props: &Props) -> Result<(), Box<error::Error>> {

        let msg = Message::new_method_call(
            BLUEZ_SERVICE, BLUEZ_OBJECT_PATH, BLUEZ_INTERFACE_ADAPTER1, BLUEZ_START_DISCOVERY)?;
        self.conn.send_with_reply_and_block(msg, 1000)?;
        let sleep_time = time::Duration::from_millis(500);
        thread::sleep(sleep_time);
        let is_discovering = match props.get("Discovering")? {
            MessageItem::Bool(b) => { b },
            _ => { panic!("Not the type that was expected!") },
        };
        if !is_discovering {
            return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other, "Can't set bluetooth to discover mode")))
        }

        Ok(())

    }

    pub fn get_managed_devices(&self) -> Result<Vec<String>, Box<error::Error>> {
        let msg = Message::new_method_call(BLUEZ_SERVICE, "/",
                                             "org.freedesktop.DBus.ObjectManager",
                                             "GetManagedObjects")?;
        let mut devices: Vec<String> = Vec::new();
        // Similar implementation as here:
        // https://github.com/szeged/blurz/blob/7729c462439fb692f12e385a84ab371423eb4cd6/src/bluetooth_utils.rs#L53
        let result = self.conn.send_with_reply_and_block(msg, 3000)?;
        let result_vec = result.get_items();
        let items: &[MessageItem] = result_vec.get(0).unwrap().inner().unwrap();
        for i in items {
            let (path, ifs) = i.inner().unwrap();
            let interfaces: &[MessageItem] = ifs.inner().unwrap();
            for intf in interfaces {
                let (intf_tmp, _) = intf.inner().unwrap();
                let intf_str: &str = intf_tmp.inner().unwrap();
                if intf_str == "org.bluez.Device1" {
                    let path_str: &str = path.inner().unwrap();
                    devices.push(path_str.to_string());
                }
            }
        }
        Ok(devices)
    }

    fn read_manufacturer_field(&self, obj_path: &str) -> Result<Option<String>, Box<error::Error>> {

        debug!("Getting manufacturer data for {}", obj_path);
        let props = Props::new(&self.conn, BLUEZ_SERVICE, obj_path,
                               BLUEZ_INTERFACE_DEVICE1, 500);
        // manufacturer data
        let prop_val = match props.get("ManufacturerData") {
            Ok(val) => val,
            Err(_) => {
                info!("No manufacturer data in {}", obj_path);
                return Ok(None);
            },
        };
        let mfr_data: &[MessageItem] = prop_val.inner().unwrap();
        for entry in mfr_data {
            let (mi_id, mi_data) = entry.inner().unwrap();
            let id: u16 = mi_id.inner().unwrap();
            debug!("manufacturer id: {}, value: ", id);
            match *mi_data {
                MessageItem::Variant(ref v) => {
                    match **v {
                        MessageItem::Array(ref a) => {
                            let arr: &[MessageItem] = a;
                            for d in arr {
                                let byte: u8 = d.inner().unwrap();
                                debug!("{}", byte);
                            }
                        },
                        _ => panic!("Not what expected"),
                    };
                },
                _ => panic!("Not what expected"),
            };

        }
        Ok(None)
    }

    pub fn get_devices(&self) -> Result<Vec<device::Device>, Box<error::Error>> {
        let devices = self.get_managed_devices()?;
        info!("Found devices {:?}", devices);
        for d in devices {
            self.read_manufacturer_field(&d)?;
        }
        Ok(Vec::new())
    }

}

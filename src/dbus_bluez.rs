use std::io::{Error, ErrorKind};
use std::error;
use std::{thread, time};
use std::collections::{HashMap, hash_map::Entry};
use dbus::{
    Message, MessageItem, MessageItemArray,
    Signature, Props, Connection, BusType,
};

use bt_sensor;
use config;
use bt_device::BTDevice;

type BoxErr = Box<error::Error>;

static BLUEZ_SERVICE: &'static str = "org.bluez";
static BLUEZ_INTERFACE_ADAPTER1: &'static str = "org.bluez.Adapter1";
static BLUEZ_START_DISCOVERY: &'static str = "StartDiscovery";
static BLUEZ_SET_DISCOVERY_FILTER: &'static str = "SetDiscoveryFilter";

pub struct DbusBluez {
    conn: Connection,
    sensor_factory: bt_sensor::BTSensorFactory,
    sensor_map: HashMap<String, Box<bt_sensor::BTSensor>>,
    bluez_obj_path: String,
    conf: config::SensorConf,
}

fn new_err(msg: &str) -> Box<Error> {
    return Box::new(Error::new(ErrorKind::Other, msg))
}

impl DbusBluez {

    pub fn new(conf: config::SensorConf, bt_devname: String) -> Result<DbusBluez, BoxErr> {
        let bluez_obj_path = format!("/org/bluez/{}", bt_devname);
        let bus = DbusBluez{
            conn: Connection::get_private(BusType::System)?,
            sensor_factory: bt_sensor::BTSensorFactory::new(conf.clone()),
            sensor_map: HashMap::new(),
            bluez_obj_path: bluez_obj_path,
            conf: conf,
        };
        Ok(bus)
    }

    pub fn initialize(&self) -> Result<(), BoxErr> {

        let props = Props::new(
            &self.conn,
            BLUEZ_SERVICE,
            &self.bluez_obj_path,
            BLUEZ_INTERFACE_ADAPTER1,
            500,
        );
        self.poweron_interface(&props)?;
        self.set_discovery_filter()?;
        self.start_discovering(&props)?;

        info!("Bluetooth discovering!");
        Ok(())

    }

    fn poweron_interface(&self, props: &Props) -> Result<(), BoxErr> {

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
                return Err(new_err("The bluetooth interface is not powered on!"))
            } else {
                info!("Interface on");
            }
        }

        Ok(())

    }

    fn set_discovery_filter(&self) -> Result<(), BoxErr> {

        let empty: Vec<MessageItem> = Vec::new();
        let str_arr_sign = Signature::new("a(s)")?;
        let uuid_arr = match MessageItemArray::new(empty, str_arr_sign) {
            Ok(a) => a,
            Err(e) => match e {
                _ => return Err(new_err("ArrayError")),
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
                _ => return Err(new_err("ArrayError")),
            },
        };

        let param = MessageItem::Array(dict_arr);

        let msg1 = Message::new_method_call(
            BLUEZ_SERVICE, &self.bluez_obj_path,
            BLUEZ_INTERFACE_ADAPTER1, BLUEZ_SET_DISCOVERY_FILTER)?.append1(param);
        self.conn.send_with_reply_and_block(msg1, 1000)?;

        Ok(())

    }

    fn start_discovering(&self, props: &Props) -> Result<(), BoxErr> {

        let msg = Message::new_method_call(
            BLUEZ_SERVICE, &self.bluez_obj_path, BLUEZ_INTERFACE_ADAPTER1, BLUEZ_START_DISCOVERY)?;
        self.conn.send_with_reply_and_block(msg, 1000)?;
        let sleep_time = time::Duration::from_millis(500);
        thread::sleep(sleep_time);
        let is_discovering = match props.get("Discovering")? {
            MessageItem::Bool(b) => { b },
            _ => { panic!("Not the type that was expected!") },
        };
        if !is_discovering {
            return Err(new_err("Can't set bluetooth to discover mode"))
        }

        Ok(())

    }

    fn read_manufacturer_data(&self, dbusmap: &MessageItem) -> Result<HashMap<u16, Vec<u8>>, BoxErr> {

        let mut map = HashMap::new();

        let map_arr: &[MessageItem] = match dbusmap.inner() {
            Ok(v) => v,
            Err(_) => return Err(new_err("inner() is not &[MessageItem]")),
        };

        for entry in map_arr {
            let (key_item, val_item) = match entry.inner() {
                Ok(v) => v,
                Err(_) => return Err(new_err("inner() is not tuple")),
            };
            let key: u16 = match key_item.inner() {
                Ok(v) => v,
                Err(_) => return Err(new_err("inner() is not u16")),
            };
            let variant = match *val_item {
                MessageItem::Variant(ref v) => v,
                _ => return Err(new_err("Not a Variant")),
            };
            let val: &[MessageItem] = match variant.inner() {
                Ok(r) => r,
                Err(_) => {
                    return Err(new_err("inner() is not &[MessageItem]"))
                },
            };
            let mut byte_arr = Vec::new();
            for entry in val {
                let byte: u8 = match entry.inner() {
                    Ok(v) => v,
                    Err(_) => return Err(new_err("Not an u8")),
                };
                byte_arr.push(byte);
            }
            map.insert(key, byte_arr);
        }

        Ok(map)

    }

    pub fn update_sensors(&mut self) -> Result<(), BoxErr> {

        let msg = Message::new_method_call(BLUEZ_SERVICE, "/",
                                             "org.freedesktop.DBus.ObjectManager",
                                             "GetManagedObjects")?;

        // Similar implementation as here:
        // https://github.com/szeged/blurz/blob/7729c462439fb692f12e385a84ab371423eb4cd6/src/bluetooth_utils.rs#L53
        let result = self.conn.send_with_reply_and_block(msg, 3000)?;
        let result_vec = result.get_items();
        let items: &[MessageItem] = result_vec.get(0).unwrap().inner().unwrap();
        for i in items {
            let (path, ifs) = i.inner().unwrap();
            let interfaces: &[MessageItem] = ifs.inner().unwrap();
            for intf in interfaces {
                let (intf_tmp, prop_map) = intf.inner().unwrap();
                let intf_str: &str = intf_tmp.inner().unwrap();
                if intf_str == "org.bluez.Device1" {
                    let path_str: &str = path.inner().unwrap();

                    let mut address = "";
                    let mut mfr_data  = HashMap::new();
                    let prop_arr: &[MessageItem] = prop_map.inner().unwrap();
                    for prop in prop_arr {
                        let (key, val) = match *prop {
                            MessageItem::DictEntry(ref k, ref v) => (k, v),
                            _ => panic!("Unexpected type!"),
                        };
                        let key_str: &str = key.inner().unwrap();
                        match key_str {
                            "Address" => {
                                let key_val: &str = match **val {
                                    MessageItem::Variant(ref v) => v.inner().unwrap(),
                                    _ => panic!("Expected Variant"),
                                };
                                info!("{:?}", key_val);
                                address = key_val;
                            },
                            "ManufacturerData" => {
                                mfr_data = match **val {
                                    MessageItem::Variant(ref v) => self.read_manufacturer_data(v)?,
                                    _ => panic!("Expected Variant"),
                                };
                            },
                            _ => continue,
                        }
                    }
                    self._update_sensor(path_str, address, mfr_data)?;
                }
            }
        }
        Ok(())

    }

    fn _update_sensor(&mut self, object_path: &str, address: &str, mfr_data: HashMap<u16, Vec<u8>>) -> Result<(), BoxErr> {

        match self.sensor_map.entry(object_path.to_string()) {
            Entry::Occupied(e) => {
                let sensor = e.into_mut();
                match sensor.get_discovery_mode() {
                    bt_sensor::DiscoveryMode::Auto => {
                        let bt_device = sensor.get_bt_device_mut();
                        bt_device.set_address(address.to_string());
                        bt_device.set_mfr_data(mfr_data);
                    },
                    bt_sensor::DiscoveryMode::Configured(_) => {
                        let bt_device = sensor.get_bt_device_mut();
                        bt_device.set_address(address.to_string());
                        bt_device.set_mfr_data(mfr_data);
                    },
                }
            },
            Entry::Vacant(e) => {
                let tag = self.conf.get_sensor_tag(address).unwrap_or(address);
                let dev = BTDevice::new(
                    object_path.to_string(),
                    address.to_string(),
                    tag.to_string(),
                    mfr_data,
                );
                match self.sensor_factory.get_sensor(dev) {
                    Some(sensor) => {
                        e.insert(sensor);
                    },
                    None => {},
                }
            }
        }

        Ok(())

    }

    pub fn get_sensors(&mut self) -> Result<&HashMap<String, Box<bt_sensor::BTSensor>>, BoxErr> {
        self.update_sensors()?;
        Ok(&self.sensor_map)
    }

}

use std::error;
use std::{thread, time::{Duration, SystemTime}};
use std::collections::{HashMap, hash_map::Entry};
use std::rc::Rc;
use std::cell::{RefCell, Ref};

use dbus::{
    Message, MessageItem, MessageItemArray,
    Signature, Props, Connection, BusType,
};

use  error::BlueZError;
use bt_sensor_factory::BTSensorFactory;
use bt_sensor::{BTSensor};
use config;
use bt_device::BTDevice;
use consumer::Consumer;

macro_rules! dbus_err {
    ($msg:expr) => {
        Box::new(BlueZError::new(format!("{}", $msg)))
    };
}

type BoxErr = Box<dyn error::Error>;

static BLUEZ_SERVICE: &'static str = "org.bluez";
static BLUEZ_INTERFACE_ADAPTER1: &'static str = "org.bluez.Adapter1";
static BLUEZ_START_DISCOVERY: &'static str = "StartDiscovery";
static BLUEZ_SET_DISCOVERY_FILTER: &'static str = "SetDiscoveryFilter";

pub struct DbusBluez {
    conn: Connection,
    sensor_factory: BTSensorFactory,
    device_map: HashMap<String, Rc<RefCell<BTDevice>>>,
    bluez_obj_path: String,
    conf: config::SensorConf,
}

impl DbusBluez {

    pub fn new(conf: config::SensorConf, bt_devname: String) -> Result<DbusBluez, BoxErr> {
        let bluez_obj_path = format!("/org/bluez/{}", bt_devname);
        let bus = DbusBluez{
            conn: Connection::get_private(BusType::System)?,
            sensor_factory: BTSensorFactory::new(conf.clone()),
            device_map: HashMap::new(),
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
            MessageItem::Bool(b) => b,
            _ => panic!("Not the type that was expected!"),
        };
        if !is_powered {
            info!("Turning the bluetooth interface on...");
            props.set("Powered", MessageItem::Bool(true))?;
            is_powered = match props.get("Powered")? {
                MessageItem::Bool(b) => b,
                _ => panic!("Not the type that was expected!"),
            };
            if !is_powered {
                return Err(dbus_err!("The bluetooth interface is not powered on!"))
            } else {
                info!("Interface on");
            }
        }

        Ok(())

    }

    fn set_discovery_filter(&self) -> Result<(), BoxErr> {

        let empty: Vec<MessageItem> = Vec::new();
        let str_arr_sign = Signature::new("a(s)")?;
        let uuid_arr = MessageItemArray::new(empty, str_arr_sign)
            .map_err(|_| dbus_err!("ArrayError"))?;
        let uuid_entry = MessageItem::DictEntry(
            Box::new(MessageItem::Str(String::from("UUIDs"))),
            Box::new(MessageItem::Variant(Box::new(MessageItem::Array(uuid_arr)))),
        );

        let transport_entry = MessageItem::DictEntry(
            Box::new(MessageItem::Str(String::from("Transport"))),
            Box::new(MessageItem::Variant(Box::new(MessageItem::Str(String::from("le"))))),
        );

        let dict_sign = Signature::new("a{sv}")?;
        let dict_arr = MessageItemArray::new(
            vec!(uuid_entry, transport_entry),
            dict_sign,
        ).map_err(|_| dbus_err!("ArrayError"))?;

        let param = MessageItem::Array(dict_arr);

        let msg1 = Message::new_method_call(
            BLUEZ_SERVICE,
            &self.bluez_obj_path,
            BLUEZ_INTERFACE_ADAPTER1,
            BLUEZ_SET_DISCOVERY_FILTER
        )?.append1(param);
        self.conn
            .send_with_reply_and_block(msg1, 1000)
            .map_err(|_| dbus_err!("DBus Error while setting discovery filter"))?;

        Ok(())

    }

    fn start_discovering(&self, props: &Props) -> Result<(), BoxErr> {

        let msg = Message::new_method_call(
            BLUEZ_SERVICE, &self.bluez_obj_path, BLUEZ_INTERFACE_ADAPTER1, BLUEZ_START_DISCOVERY)?;
        self.conn.send_with_reply_and_block(msg, 1000)?;
        let sleep_time = Duration::from_millis(500);
        thread::sleep(sleep_time);
        let is_discovering = match props.get("Discovering")? {
            MessageItem::Bool(b) => b,
            _ => panic!("Not the type that was expected!"),
        };
        if !is_discovering {
            return Err(dbus_err!("Can't set bluetooth to discover mode"))
        }

        Ok(())

    }

    fn read_manufacturer_data(&self, dbusmap: &MessageItem) -> Result<HashMap<u16, Vec<u8>>, BoxErr> {

        let mut map = HashMap::new();

        let map_arr: &[MessageItem] = dbusmap
            .inner()
            .map_err(|_| dbus_err!("inner() is not &[MessageItem]"))?;

        for entry in map_arr {
            let (key_item, val_item) = entry
                .inner()
                .map_err(|_| dbus_err!("inner() is not tuple"))?;
            let key: u16 = key_item
                .inner()
                .map_err(|_| dbus_err!("inner() is not u16"))?;
            let variant = match *val_item {
                MessageItem::Variant(ref v) => v,
                _ => return Err(dbus_err!("Not a Variant")),
            };
            let val: &[MessageItem] = variant
                .inner()
                .map_err(|_| dbus_err!("inner() is not &[MessageItem]"))?;
            let mut byte_arr = Vec::new();
            for entry in val {
                let byte: u8 = entry
                    .inner()
                    .map_err(|_| dbus_err!("Not an u8"))?;
                byte_arr.push(byte);
            }
            map.insert(key, byte_arr);
        }

        Ok(map)

    }

    fn read_service_data(&self, dbusmap: &MessageItem) -> Result<HashMap<String, Vec<u8>>, BoxErr> {

        let mut map = HashMap::new();

        let map_arr: &[MessageItem] = dbusmap
            .inner()
            .map_err(|_| dbus_err!("inner() is not &[MessageItem]"))?;

        for entry in map_arr {
            let (key_item, val_item) = entry
                .inner()
                .map_err(|_| dbus_err!("inner() is not tuple"))?;
            let key: &str = key_item
                .inner()
                .map_err(|_| dbus_err!("inner() is not &str"))?;
            let variant = match *val_item {
                MessageItem::Variant(ref v) => v,
                _ => return Err(dbus_err!("Not a Variant")),
            };
            let val: &[MessageItem] = variant
                .inner()
                .map_err(|_| dbus_err!("inner() is not &[MessageItem]"))?;
            let mut byte_arr = Vec::new();
            for entry in val {
                let byte: u8 = entry
                    .inner()
                    .map_err(|_| dbus_err!("Not an u8"))?;
                byte_arr.push(byte);
            }
            map.insert(key.to_string(), byte_arr);
        }

        Ok(map)

    }

    pub fn update_sensors(&mut self) -> Result<(), BoxErr> {

        let msg = Message::new_method_call(
            BLUEZ_SERVICE,
            "/",
            "org.freedesktop.DBus.ObjectManager",
            "GetManagedObjects",
        )?;

        // Similar implementation as here:
        // https://github.com/szeged/blurz/blob/7729c462439fb692f12e385a84ab371423eb4cd6/src/bluetooth_utils.rs#L53
        let result = self.conn
            .send_with_reply_and_block(msg, 3000)
            .map_err(|_| dbus_err!("Failed to make dbus query".to_string()))?;
        let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(t) => t,
            Err(_) => panic!("System clock before unix epoch!"),
        };
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
                    let mut mfr_data = None;
                    let mut svc_data = None;
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
                                debug!("{:?}", key_val);
                                address = key_val;
                            },
                            "ManufacturerData" => {
                                mfr_data = match **val {
                                    MessageItem::Variant(ref v) => Some(self.read_manufacturer_data(v)?),
                                    _ => panic!("Expected Variant"),
                                };
                            },
                            "ServiceData" => {
                                svc_data = match **val {
                                    MessageItem::Variant(ref v) => Some(self.read_service_data(v)?),
                                    _ => panic!("Expected Variant"),
                                };
                            },
                            _ => continue,
                        }
                    }
                    let millis = timestamp.subsec_millis() as u64;
                    let unix_ts = timestamp.as_secs() * 1000 + millis;
                    self._update_device(path_str, address, mfr_data, svc_data, unix_ts)?;
                }
            }
        }
        Ok(())

    }

    fn _update_device(
        &mut self,
        object_path: &str,
        address: &str,
        mfr_data: Option<HashMap<u16, Vec<u8>>>,
        svc_data: Option<HashMap<String, Vec<u8>>>,
        meas_timestamp: u64,
        ) -> Result<(), BoxErr>
    {

        let tag = self.conf.get_sensor_tag(address).unwrap_or(address);
        match self.device_map.entry(object_path.to_string()) {
            Entry::Occupied(mut e) => {
                let mut device = e.get_mut();
                device.borrow_mut().update_data(mfr_data, svc_data, meas_timestamp);
                self.sensor_factory.set_sensor(device.clone());
            },
            Entry::Vacant(e) => {
                let device = Rc::new(RefCell::new(BTDevice::new(
                    object_path.to_string(),
                    address.to_string(),
                    tag.to_string(),
                    mfr_data,
                    svc_data,
                    meas_timestamp,
                    self.conf.get_last_seen_forget(),
                    self.sensor_factory.get_sensor_discovery_mode(address),
                )));
                self.sensor_factory.set_sensor(device.clone());
                if device.borrow().get_sensor().is_some() {
                    e.insert(device);
                }
            }
        };

        Ok(())

    }

    pub fn consume(&mut self, consumer: &mut dyn Consumer) -> Result<(), BoxErr> {
        self.update_sensors()?;
        let devices: Vec<Ref<BTDevice>> = self.device_map.iter()
            .map(|(_, d)| d.borrow())
            .collect();
        let sensors: Vec<&dyn BTSensor> = devices.iter()
            .filter_map(|d| d.get_sensor())
            .collect();
        consumer.consume(&sensors);
        Ok(())
    }

}

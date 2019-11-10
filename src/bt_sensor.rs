use std::rc::Rc;
use std::cell::{RefCell, Ref};
use std::collections::HashMap;

use bt_device::BTDevice;
use discovery_mode::DiscoveryMode;

pub trait BTSensor {

    fn is_valid_data(&self) -> bool;

    fn get_measurements(&self) -> Option<HashMap<String, Value>>;
    fn get_measurements_json_str(&self) -> Option<String>;
    fn get_measurements_str(&self) -> Option<String>;

    fn get_bt_device(&self) -> Ref<BTDevice>;

    fn get_measurement_timestamp(&self) -> u64;
    fn get_address(&self) -> String;
    fn get_tag(&self) -> String;

}

pub trait BTSensorConstructor {
    fn get_name(&self) -> &'static str;
    fn construct(&self, device: Rc<RefCell<BTDevice>>, discovery_mode: DiscoveryMode) -> Box<dyn BTSensor>;
    fn is_valid_data(&self, device: &BTDevice) -> bool;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value {
	String(String),
	Integer(i64),
	Float(f64),
	Boolean(bool),
}

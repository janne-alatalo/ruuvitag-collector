use std::collections::HashMap;

use bt_device::BTDevice;

#[derive(Debug, Clone)]
pub enum DiscoveryMode {
    Auto,
    Configured(String),
}

impl Default for DiscoveryMode {
    fn default() -> DiscoveryMode { DiscoveryMode::Auto }
}

pub trait BTSensor {

    fn is_valid_data(&self, device: &BTDevice) -> bool;

    fn get_measurements(&self) -> Option<HashMap<String, Value>>;
    fn get_measurements_json_str(&self) -> Option<String>;
    fn get_measurements_str(&self) -> Option<String>;
    fn get_discovery_mode(&self) -> &DiscoveryMode;

    fn get_bt_device(&self) -> &BTDevice;
    fn get_bt_device_mut(&mut self) -> &mut BTDevice;

    fn get_measurement_timestamp(&self) -> u64;
    fn get_address(&self) -> &str;
    fn get_tag(&self) -> &str;

    fn set_device(&mut self, bt_device: BTDevice);

}

pub trait BTSensorConstructor {
    fn get_name(&self) -> &'static str;
    fn construct(&self, device: BTDevice, discovery_mode: DiscoveryMode) -> Box<BTSensor>;
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

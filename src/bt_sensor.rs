use std::collections::HashMap;

use bt_device::BTDevice;

pub type SensorIFConstr = Box<Fn(BTDevice) -> Box<BTSensor>>;


#[derive(Debug)]
pub enum DiscoveryMode {
    Auto,
    Configured(String),
}

impl Default for DiscoveryMode {
    fn default() -> DiscoveryMode { DiscoveryMode::Auto }
}

pub trait BTSensor {

    fn is_valid_data(&self, device: &BTDevice) -> bool;

    fn get_measurements(&self) -> Option<HashMap<String, i32>>;
    fn get_measurements_json_str(&self) -> Option<String>;
    fn get_discovery_mode(&self) -> &DiscoveryMode;

    fn get_bt_device(&self) -> &BTDevice;
    fn get_bt_device_mut(&mut self) -> &mut BTDevice;

    fn reset_last_seen(&mut self);
    fn get_last_seen(&mut self);

    fn get_address(&self) -> &str;

}

use std::collections::HashMap;

use config;
use bt_sensor::BTSensorConstructor;
use ruuvitag_df3::RuuvitagDF3Constructor;
use ruuvitag_df2::RuuvitagDF2Constructor;
use bt_device::BTDevice;
use bt_sensor::BTSensor;

pub struct BTSensorFactory {
    conf: config::SensorConf,
    sensor_constructors: HashMap<&'static str, Box<BTSensorConstructor>>,
}

impl BTSensorFactory {

    pub fn new(conf: config::SensorConf) -> BTSensorFactory {
        let mut factory = BTSensorFactory{
            conf,
            sensor_constructors: HashMap::new(),
        };
        factory.initialize();
        factory
    }

    pub fn initialize(&mut self) {
        self.init_sensor_constructors();
    }

    fn init_sensor_constructors(&mut self) {
        let constr = RuuvitagDF3Constructor::new();
        self.sensor_constructors.insert(constr.get_name(), constr);
        let constr = RuuvitagDF2Constructor::new();
        self.sensor_constructors.insert(constr.get_name(), constr);
    }

    pub fn get_sensor(&self, bt_device: BTDevice) -> Option<Box<BTSensor>> {
        match self.conf.get_sensor_if(bt_device.get_address()) {
            Some(sensor_if) => self.get_sensor_type(sensor_if, bt_device),
            None => None,
        }
    }

    fn get_sensor_type(&self, sensor_type: &str, bt_device: BTDevice) -> Option<Box<BTSensor>> {
        match self.sensor_constructors.get(sensor_type) {
            Some(constructor) => Some(constructor.construct(bt_device)),
            None => None,
        }
    }

    fn auto_discover(&self, object_path: String, address: String, mfr_data: HashMap<u16, Vec<u8>>) -> Option<&'static str> {
        None
    }

}


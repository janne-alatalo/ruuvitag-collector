use std::collections::HashMap;

use config;
use bt_sensor::SensorIFConstr;
use ruuvitag_df3::RuuvitagDF3;
use ruuvitag_df2::RuuvitagDF2;
use bt_device::BTDevice;
use bt_sensor::BTSensor;

pub struct BTSensorFactory {
    conf: config::SensorConf,
    sensor_constructors: HashMap<&'static str, SensorIFConstr>,
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
        let (key, constr_func) = RuuvitagDF3::get_sensor_if_constructor();
        self.sensor_constructors.insert(key, constr_func);
        let (key, constr_func) = RuuvitagDF2::get_sensor_if_constructor();
        self.sensor_constructors.insert(key, constr_func);
    }

    pub fn get_sensor(&self, bt_device: BTDevice) -> Option<Box<BTSensor>> {
        match self.conf.get_sensor_if(bt_device.get_address()) {
            Some(sensor_if) => self.get_sensor_type(sensor_if, bt_device),
            None => None,
        }
    }

    fn get_sensor_type(&self, sensor_type: &str, bt_device: BTDevice) -> Option<Box<BTSensor>> {
        match self.sensor_constructors.get(sensor_type) {
            Some(constructor) => Some(constructor(bt_device)),
            None => None,
        }
    }

    fn auto_discover(&self, object_path: String, address: String, mfr_data: HashMap<u16, Vec<u8>>) -> Option<&'static str> {
        None
    }

}


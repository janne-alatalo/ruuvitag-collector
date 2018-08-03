use std::collections::HashMap;

use config;
use bt_sensor::BTSensorConstructor;
use ruuvitag_df3::RuuvitagDF3Constructor;
use ruuvitag_df2::RuuvitagDF2Constructor;
use bt_device::BTDevice;
use bt_sensor::{BTSensor, DiscoveryMode};

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
            None => {
                if self.conf.is_auto() {
                    return self.autofind_sensor_type(&bt_device)
                }
                None
            },
        }
    }

    fn get_sensor_type(&self, sensor_type: &str, bt_device: BTDevice) -> Option<Box<BTSensor>> {
        if sensor_type == "auto" {
            return self.autofind_sensor_type(&bt_device)
        }
        match self.sensor_constructors.get(sensor_type) {
            Some(constructor) => {
                let name = constructor.get_name().to_string();
                Some(constructor.construct(bt_device, DiscoveryMode::Configured(name)))
            },
            None => None,
        }
    }

    pub fn auto_discover(&self, sensor: &mut Box<BTSensor>, device: BTDevice) -> Option<Box<BTSensor>> {
        match sensor.is_valid_data(&device) {
            true => {
                sensor.set_device(device);
                None
            },
            false => {
                match self.autofind_sensor_type(&device) {
                    Some(sensor) => Some(sensor),
                    None => {
                        sensor.set_device(device);
                        None
                    },
                }
            }
        }
    }

    fn autofind_sensor_type(&self, bt_device: &BTDevice) -> Option<Box<BTSensor>> {
        for (_, v) in &self.sensor_constructors {
            match v.is_valid_data(bt_device) {
                true => {
                    return Some(
                        v.construct(
                            bt_device.clone(),
                            DiscoveryMode::Auto
                        )
                    )
                },
                false => {}
            }
        }
        None
    }

}


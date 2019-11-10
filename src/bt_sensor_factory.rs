use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use config;
use bt_sensor::BTSensorConstructor;
use ruuvitag_df3::RuuvitagDF3Constructor;
use ruuvitag_df2::RuuvitagDF2Constructor;
use bt_device::BTDevice;
use bt_sensor::{BTSensor};
use discovery_mode::DiscoveryMode;

pub struct BTSensorFactory {
    conf: config::SensorConf,
    sensor_constructors: HashMap<&'static str, Box<dyn BTSensorConstructor>>,
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

    pub fn get_sensor_discovery_mode(&self, address: &str) -> DiscoveryMode {
        match self.conf.get_sensor_if(address) {
            Some(sensor_if) => DiscoveryMode::Configured(sensor_if.to_string()),
            None => DiscoveryMode::Auto,
        }
    }

    pub fn set_sensor(&self, bt_device: Rc<RefCell<BTDevice>>) {
        let discovery_mode = bt_device.borrow().get_discovery_mode();
        match discovery_mode {
            DiscoveryMode::Configured(ref sensor_if) => {
                if bt_device.borrow().get_sensor().is_none() {
                    if sensor_if == "auto" {
                        let is_valid_data = bt_device
                            .borrow()
                            .get_sensor()
                            .map_or(false, |s| s.is_valid_data());
                        if is_valid_data {
                            return;
                        }
                    }
                    let sensor_type = self.get_sensor_type(sensor_if, bt_device.clone());
                    if let Some(sensor) = sensor_type {
                        bt_device.borrow_mut().set_sensor(sensor);
                    } else {
                        bt_device.borrow_mut().unset_sensor();
                    }
                }
            },
            DiscoveryMode::Auto => {
                // Check if the already configured sensor type is still valid for this device
                let is_valid_data = bt_device
                    .borrow()
                    .get_sensor()
                    .map_or(false, |s| s.is_valid_data());
                if is_valid_data {
                    return;
                }
                if self.conf.is_auto() {
                    let sensor_type = self.autofind_sensor_type(bt_device.clone());
                    if let Some(sensor) = sensor_type {
                        bt_device.borrow_mut().set_sensor(sensor);
                    } else {
                        bt_device.borrow_mut().unset_sensor();
                    }
                }
            },
        }
    }

    fn get_sensor_type(&self, sensor_type: &str, bt_device: Rc<RefCell<BTDevice>>) -> Option<Box<dyn BTSensor>> {
        if sensor_type == "auto" {
            return self.autofind_sensor_type(bt_device)
        }
        match self.sensor_constructors.get(sensor_type) {
            Some(constructor) => {
                let name = constructor.get_name().to_string();
                Some(constructor.construct(bt_device, DiscoveryMode::Configured(name)))
            },
            None => None,
        }
    }

    fn autofind_sensor_type(&self, bt_device: Rc<RefCell<BTDevice>>) -> Option<Box<dyn BTSensor>> {
        for (_, v) in &self.sensor_constructors {
            let is_valid_data = v.is_valid_data(&bt_device.borrow());
            match is_valid_data {
                true => {
                    return Some(
                        v.construct(
                            bt_device,
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


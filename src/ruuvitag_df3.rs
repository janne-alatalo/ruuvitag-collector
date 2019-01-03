use std::collections::HashMap;

use serde_json;

use bt_sensor::{DiscoveryMode, BTSensor, BTSensorConstructor, Value};
use bt_device::BTDevice;

pub struct RuuvitagDF3Constructor;

impl RuuvitagDF3Constructor {
    pub fn new() -> Box<RuuvitagDF3Constructor> {
        Box::new(RuuvitagDF3Constructor{})
    }
}

impl BTSensorConstructor for RuuvitagDF3Constructor {
    fn get_name(&self) -> &'static str {
        "RuuvitagDF3"
    }
    fn construct(&self, device: BTDevice, discovery_mode: DiscoveryMode) -> Box<BTSensor> {
        Box::new(RuuvitagDF3::new(device, discovery_mode))
    }
    fn is_valid_data(&self, device: &BTDevice) -> bool {
        RuuvitagDF3::_is_valid_data(device)
    }
}

#[derive(Default, Debug)]
pub struct RuuvitagDF3 {
    last_seen: i32,
    discovery_mode: DiscoveryMode,
    bt_device: BTDevice,
}

impl BTSensor for RuuvitagDF3 {

    fn is_valid_data(&self, device: &BTDevice) -> bool {
        RuuvitagDF3::_is_valid_data(device)
    }

    fn get_measurements_json_str(&self) -> Option<String> {
        self._get_measurements_json_str()
    }

    fn get_measurements(&self) -> Option<HashMap<String, Value>> {
        match self._get_measurements() {
            Some(m) => {
                let mut map = HashMap::<String, Value>::new();
                map.insert(
                    "battery".to_string(),
                    Value::Integer(m.battery as i64),
                );
                map.insert(
                    "humidity".to_string(),
                    Value::Integer(m.humidity as i64),
                );
                map.insert(
                    "temperature".to_string(),
                    Value::Float(m.get_temperature_float()),
                );
                map.insert(
                    "pressure".to_string(),
                    Value::Integer(m.pressure as i64),
                );
                map.insert(
                    "acceleration_x".to_string(),
                    Value::Integer(m.acceleration_x as i64),
                );
                map.insert(
                    "acceleration_y".to_string(),
                    Value::Integer(m.acceleration_y as i64),
                );
                map.insert(
                    "acceleration_z".to_string(),
                    Value::Integer(m.acceleration_z as i64),
                );
                Some(map)
            },
            None => None,
        }
    }

    fn get_discovery_mode(&self) -> &DiscoveryMode {
        &self.discovery_mode
    }

    fn reset_last_seen(&mut self) {
        self.last_seen = 0;
    }

    fn get_last_seen(&mut self) {
        self.last_seen += self.last_seen;
    }

    fn get_bt_device(&self) -> &BTDevice {
        &self.bt_device
    }

    fn get_bt_device_mut(&mut self) -> &mut BTDevice {
        &mut self.bt_device
    }

    fn get_address(&self) -> &str {
        self.get_bt_device().get_address()
    }

    fn get_tag(&self) -> &str {
        self.get_bt_device().get_tag()
    }

    fn get_measurement_timestamp(&self) -> u64 {
        self.get_bt_device().get_measurement_timestamp()
    }

    fn set_device(&mut self, bt_device: BTDevice) {
        self.bt_device = bt_device;
    }

}

static MFR_DATA_FIELD: u16 = 0x0499;

impl RuuvitagDF3 {

    pub fn new(bt_device: BTDevice, discovery_mode: DiscoveryMode) -> RuuvitagDF3 {
        RuuvitagDF3{bt_device, discovery_mode, ..Default::default()}
    }

    pub fn _is_valid_data(device: &BTDevice) -> bool {
        match device.get_mfr_data().and_then(|m| m.get(&MFR_DATA_FIELD)) {
            Some(data) => {
                if data.len() == 18 {
                    return true;
                }
                false
            },
            None => {
                false
            }
        }
    }

    // See https://github.com/ruuvi/ruuvi-sensor-protocols#data-format-3-protocol-specification
    // for the specification
    pub fn get_data_format(&self) -> Option<u8> {
        self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(0)
            .map(|v| *v)
    }

    pub fn get_humidity(&self) -> Option<u8> {
        self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(1)
            .map(|v| *v)
    }

    pub fn get_temp_wholes(&self) -> Option<u8> {
        self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(2)
            .map(|raw_temp| {
                (0x7F & raw_temp)
            })
    }

    pub fn get_temp_sign(&self) -> Option<i8> {
        self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(2)
            .map(|raw_temp| {
                match raw_temp & 0x80 {
                    0 => 1,
                    _ => -1,
                }
            })
    }

    pub fn get_temp_fractions(&self) -> Option<u8> {
        self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(3)
            .map(|v| *v)
    }

    pub fn get_temp_float(&self) -> Option<f64> {
        self._get_measurements().map(|m| m.get_temperature_float())
    }

    pub fn get_pressure(&self) -> Option<u16> {
        let pressure_top = self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(4)?;
        let pressure_bottom = self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(5)?;
        Some(((*pressure_top as u16) << 8) | *pressure_bottom as u16)
    }

    pub fn get_acceleration_x(&self) -> Option<i16> {
        let acc_x_top = self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(6)?;
        let acc_x_bottom = self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(7)?;
        Some((((*acc_x_top as u16) << 8) | *acc_x_bottom as u16) as i16)
    }

    pub fn get_acceleration_y(&self) -> Option<i16> {
        let acc_y_top = self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(8)?;
        let acc_y_bottom = self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(9)?;
        Some((((*acc_y_top as u16) << 8) | *acc_y_bottom as u16) as i16)
    }

    pub fn get_acceleration_z(&self) -> Option<i16> {
        let acc_z_top = self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(10)?;
        let acc_z_bottom = self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(11)?;
        Some((((*acc_z_top as u16) << 8) | *acc_z_bottom as u16) as i16)
    }

    pub fn get_battery(&self) -> Option<u16> {
        let batt_top = self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(12)?;
        let batt_bottom = self.bt_device
            .get_mfr_data()?
            .get(&MFR_DATA_FIELD)?
            .get(13)?;
        Some(((*batt_top as u16) << 8) | *batt_bottom as u16)
    }

    pub fn get_status(&self) -> String {
        self._get_measurements()
            .map(|m| {
                format!(
                    "battery {}\ntemp {}°C\thumidity {:.1}%\tpressure {} Pa\nacc-x {}\tacc-y {}\tacc-z {}",
                    m.battery,
                    m.get_temperature_float(),
                    m.humidity,
                    m.pressure,
                    m.acceleration_x,
                    m.acceleration_y,
                    m.acceleration_z,
                )
            })
            .unwrap_or("Invalid manufacturer data".to_string())
    }

    fn _get_measurements_json_str(&self) -> Option<String> {
        match self._get_measurements() {
            Some(meas) => {
                serde_json::to_string(&meas).ok()
            },
            None => None,
        }
    }

    fn _get_measurements(&self) -> Option<RuuvitagDF3Meas> {
        if let (
            Some(format), Some(hum), Some(temp_wholes), Some(temp_sign),
            Some(temp_fract), Some(press), Some(acc_x),
            Some(acc_y), Some(acc_z), Some(batt)) = (
            self.get_data_format(), self.get_humidity(), self.get_temp_wholes(), self.get_temp_sign(),
            self.get_temp_fractions(), self.get_pressure(), self.get_acceleration_x(),
            self.get_acceleration_y(), self.get_acceleration_z(), self.get_battery()) {

            let press_corr = 50000 + press as u32;

            let tag = self.bt_device.get_tag().to_string();
            let address = self.bt_device.get_address().to_string();

            let meas = RuuvitagDF3Meas{
                data_format: format,
                battery: batt,
                humidity: hum,
                temperature: temp_wholes,
                temperature_sign: temp_sign,
                temperature_fractions: temp_fract,
                pressure: press_corr,
                acceleration_x: acc_x,
                acceleration_y: acc_y,
                acceleration_z: acc_z,
                address: address,
                tag: tag,
            };
            Some(meas)
        } else {
            None
        }
    }

}

#[derive(Default, Debug, Serialize)]
pub struct RuuvitagDF3Meas {
    data_format: u8,
    battery: u16,
    humidity: u8,
    temperature: u8,
    temperature_sign: i8,
    temperature_fractions: u8,
    pressure: u32,
    acceleration_x: i16,
    acceleration_y: i16,
    acceleration_z: i16,
    address: String,
    tag: String,
}

impl RuuvitagDF3Meas {
    fn get_temperature_float(&self) -> f64 {
        (self.temperature as f64 + (self.temperature_fractions as f64 / 100.0)) * self.temperature_sign as f64
    }
}

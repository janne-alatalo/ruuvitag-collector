use std::str;
use std::collections::HashMap;

use base64;
use serde_json;

use bt_sensor::{DiscoveryMode, BTSensor, BTSensorConstructor, Value};
use bt_device::BTDevice;

pub struct RuuvitagDF2Constructor;

impl RuuvitagDF2Constructor {
    pub fn new() -> Box<RuuvitagDF2Constructor> {
        Box::new(RuuvitagDF2Constructor{})
    }
}

impl BTSensorConstructor for RuuvitagDF2Constructor {
    fn get_name(&self) -> &'static str {
        "RuuvitagDF2"
    }
    fn construct(&self, device: BTDevice, discovery_mode: DiscoveryMode) -> Box<BTSensor> {
        Box::new(RuuvitagDF2::new(device, discovery_mode))
    }
    fn is_valid_data(&self, device: &BTDevice) -> bool {
        RuuvitagDF2::_is_valid_data(device)
    }
}

#[derive(Default, Debug)]
pub struct RuuvitagDF2 {
    last_seen: i32,
    discovery_mode: DiscoveryMode,
    bt_device: BTDevice,
}

impl BTSensor for RuuvitagDF2 {

    fn is_valid_data(&self, device: &BTDevice) -> bool {
        RuuvitagDF2::_is_valid_data(device)
    }

    fn get_measurements_str(&self) -> Option<String> {
        self._get_measurements_str()
    }

    fn get_measurements_json_str(&self) -> Option<String> {
        self._get_measurements_json_str()
    }

    fn get_measurements(&self) -> Option<HashMap<String, Value>> {
        match self._get_measurements() {
            Some(m) => {
                let mut map = HashMap::<String, Value>::new();
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

static SVC_DATA_UUID: &'static str = "0000feaa-0000-1000-8000-00805f9b34fb";

impl RuuvitagDF2 {

    pub fn new(bt_device: BTDevice, discovery_mode: DiscoveryMode) -> RuuvitagDF2 {
        RuuvitagDF2{bt_device, discovery_mode, ..Default::default()}
    }

    pub fn _is_valid_data(device: &BTDevice) -> bool {
        match device.get_svc_data().and_then(|m| m.get(SVC_DATA_UUID)) {
            Some(data) => {
                if data.len() == 20 {
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
    pub fn get_data_format(data: &Vec<u8>) -> Option<u8> {
        data.get(0).map(|v| *v)
    }

    pub fn get_humidity(data: &Vec<u8>) -> Option<f32> {
        let humidity = data.get(1).map(|v| *v)?;
        Some((humidity as f32) * 0.5_f32)
    }

    pub fn get_temp_wholes(data: &Vec<u8>) -> Option<u8> {
        data.get(2).map(|u8_temp| {
                (0x7F & u8_temp)
            })
    }

    pub fn get_temp_sign(data: &Vec<u8>) -> Option<i8> {
        data.get(2).map(|raw_temp| {
                match raw_temp & 0x80 {
                    0 => 1,
                    _ => -1,
                }
            })
    }

    pub fn get_temp_fractions(data: &Vec<u8>) -> Option<u8> {
        data.get(3).map(|v| *v)
    }

    pub fn get_pressure(data: &Vec<u8>) -> Option<u16> {
        let pressure_top = data.get(4)?;
        let pressure_bottom = data.get(5)?;
        Some(((*pressure_top as u16) << 8) | *pressure_bottom as u16)
    }


    pub fn get_id(data: &Vec<u8>) -> Option<u8> {
        data.get(6).map(|v| *v)
    }

    fn _get_measurements(&self) -> Option<RuuvitagDF2Meas> {
        let data_vec = self.bt_device
            .get_svc_data()?
            .get(SVC_DATA_UUID)?;

        if data_vec.len() < 4 {
            return None
        }

        let slice = &data_vec[3..];
        let uri = str::from_utf8(slice).ok()?;
        let cuts = uri.split("#").collect::<Vec<&str>>();
        // Not really sure about this... apparently the string is too short and the base64 package
        // cant decode it. I am pretty sure this fixes it.
        let fixed_len = format!("{}A", cuts.get(1)?);
        let data = base64::decode_config(&fixed_len, base64::STANDARD_NO_PAD).ok()?;

        if let (
            Some(format), Some(hum), Some(temp_wholes), Some(temp_fract), Some(temp_sign),
            Some(press), Some(id)) = (
            RuuvitagDF2::get_data_format(&data), RuuvitagDF2::get_humidity(&data),
            RuuvitagDF2::get_temp_wholes(&data), RuuvitagDF2::get_temp_fractions(&data),
            RuuvitagDF2::get_temp_sign(&data), RuuvitagDF2::get_pressure(&data),
            RuuvitagDF2::get_id(&data)) {

            let press_corr = 50000 + press as u32;

            let tag = self.bt_device.get_tag().to_string();
            let address = self.bt_device.get_address().to_string();

            let meas = RuuvitagDF2Meas{
                data_format: format,
                humidity: hum,
                temperature: temp_wholes,
                temperature_sign: temp_sign,
                temperature_fractions: temp_fract,
                pressure: press_corr,
                id: id,
                address: address,
                tag: tag,
            };
            Some(meas)
        } else {
            None
        }
    }

    pub fn _get_measurements_str(&self) -> Option<String> {
        self._get_measurements()
            .map(|m| {
                format!(
                    "temp {}°C\thumidity {:.1}%\tpressure {} Pa\nid {}\n",
                    m.get_temperature_float(),
                    m.humidity,
                    m.pressure,
                    m.id,
                )
            })
    }

    fn _get_measurements_json_str(&self) -> Option<String> {
        match self._get_measurements() {
            Some(meas) => serde_json::to_string(&meas).ok(),
            None => None
        }
    }

}

#[derive(Default, Debug, Serialize)]
pub struct RuuvitagDF2Meas {
    data_format: u8,
    humidity: f32,
    temperature: u8,
    temperature_sign: i8,
    temperature_fractions: u8,
    pressure: u32,
    id: u8,
    address: String,
    tag: String,
}

impl RuuvitagDF2Meas {
    fn get_temperature_float(&self) -> f64 {
        (self.temperature as f64 + (self.temperature_fractions as f64 / 100.0)) * self.temperature_sign as f64
    }
}

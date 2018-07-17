use std::collections::HashMap;
use std::fs::File;

use serde_json;

pub struct SensorInfo {
    id: String,
    address: String,
    sensor_if: String,
}

impl SensorInfo {

    pub fn new(address: String, id: String, sensor_if: String) -> SensorInfo {
        SensorInfo{
            id, address, sensor_if,
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_sensor_if(&self) -> &str {
        &self.sensor_if
    }

}

#[derive(Default)]
pub struct SensorConf {
    auto: bool,
    address_map: HashMap<String, SensorInfo>,
}

impl SensorConf {

    pub fn new(devicemap_file: Option<String>) -> SensorConf {
        let address_map = match devicemap_file {
            Some(f) => SensorConf::parse_devicemap_file(f),
            None => HashMap::new(),
        };
        SensorConf{
            auto: true,
            address_map: address_map,
        }
    }

    fn parse_devicemap_file(filename: String) -> HashMap<String, SensorInfo> {

        let f = File::open(&filename)
            .expect(&format!("Cannot open file {}", filename));
        let v: serde_json::Value = serde_json::from_reader(f)
            .map_err(|e| panic!("JSON error in {}: {}", filename, e))
            .unwrap();

        v.as_object()
            .expect(&format!("Invalid JSON in {}, not an object", filename))
            .iter()
            .map(|(k, v)| {
                let val = v.as_object().expect(&format!("Value not an object in {}", filename));
                let address = k;
                let id = val
                    .get("id")
                    .map(|id|
                         id.as_str().expect(&format!("Id not string in {}, device {}", filename, address))
                     )
                    .unwrap_or(address);
                let sensor_if = val
                    .get("sensor_if")
                    .map(|parser|
                         parser.as_str().expect(&format!("sensor_if not string in {}, device {}", filename, address))
                     )
                    .unwrap_or("auto");
                (
                    k.to_string(),
                    SensorInfo::new(
                        id.to_string(),
                        address.to_string(),
                        sensor_if.to_string(),
                    )
                )
            })
            .collect()

    }

    pub fn is_auto(&self) -> bool {
        self.auto
    }

    pub fn get_sensor_if(&self, address: &str) -> Option<&str> {
        self.address_map
            .get(address)
            .map(|c| c.get_sensor_if())
    }

}

use std::collections::HashMap;
use std::fs::File;

use serde_json;

use ::Args;

#[derive(Clone)]
pub struct SensorInfo {
    address: String,
    tag: String,
    sensor_if: String,
}

impl SensorInfo {

    pub fn new(address: String, tag: String, sensor_if: String) -> SensorInfo {
        SensorInfo{
            address, tag, sensor_if,
        }
    }

    pub fn get_tag(&self) -> &str {
        &self.tag
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_sensor_if(&self) -> &str {
        &self.sensor_if
    }

}

#[derive(Default, Clone)]
pub struct SensorConf {
    auto: bool,
    address_map: HashMap<String, SensorInfo>,
}

impl SensorConf {

    pub fn new(args: &Args) -> SensorConf {

        let address_map = match args.flag_devicemap {
            Some(ref f) => SensorConf::parse_devicemap_file(f),
            None => HashMap::new(),
        };
        SensorConf{
            auto: true,
            address_map: address_map,
        }
    }

    fn parse_devicemap_file(filename: &str) -> HashMap<String, SensorInfo> {

        let f = File::open(filename)
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
                let tag = val
                    .get("tag")
                    .map(|tag|
                         tag.as_str().expect(&format!("tag not string in {}, device {}", filename, address))
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
                        address.to_string(),
                        tag.to_string(),
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

    pub fn get_sensor_tag(&self, address: &str) -> Option<&str> {
        self.address_map
            .get(address)
            .map(|c| c.get_tag())
    }

}

use std::collections::HashMap;
use std::env;

use serde_json;
use influx_db_client::{Client};

use bt_sensor::Value;

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub enum ConsumerType {
    StdOut,
    Influxdb,
}

pub trait Consumer {
    fn consume(&self, measurement: &HashMap<String, Value>);
}

pub fn initialize_consumer(consumer_name: &ConsumerType) -> Result<Box<Consumer>, String> {
    match consumer_name {
        ConsumerType::StdOut => {
            Ok(Box::new(StdOutConsumer{}))
        },
        _ => {
            Err(format!("No consumer {:?}", consumer_name))
        }
    }
}

pub struct StdOutConsumer;

impl Consumer for StdOutConsumer {
    fn consume(&self, measurement: &HashMap<String, Value>) {
        let json = serde_json::to_string(&measurement)
            .expect("Failed to serialize measurement to json");
        println!("{}", json);
    }
}

pub struct InfluxdbConsumer {
    client: Client,
}

impl InfluxdbConsumer {
    fn new() -> InfluxdbConsumer {
        let influx_url = env::var_os("INFLUXDB_URL")
            .map(|s| s.to_str().expect("INFLUXDB_URL conversion error").to_string())
            .unwrap_or("http://127.0.0.1:8086".into());
        let influx_db = env::var_os("INFLUXDB_DB")
            .map(|s| s.to_str().expect("INFLUXDB_DB conversion error").to_string())
            .unwrap_or("ruuvitag".into());
        let client = Client::new(influx_url, influx_db);
        InfluxdbConsumer{client}
    }
}

impl Consumer for InfluxdbConsumer {
    fn consume(&self, measurement: &HashMap<String, Value>) {
        let json = serde_json::to_string(&measurement)
            .expect("Failed to serialize measurement to json");
        println!("{}", json);
    }
}

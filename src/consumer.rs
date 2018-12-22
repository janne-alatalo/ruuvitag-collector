use std::collections::HashMap;
use std::env;

use serde_json;
use influx_db_client::{Client, Point, Value as InfluxVal, Precision};

use bt_sensor::{Value, BTSensor};

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub enum ConsumerType {
    StdOut,
    Influxdb,
}

pub trait Consumer {
    fn consume(&self, sensor: &Box<BTSensor>, measurement: &HashMap<String, Value>);
}

pub fn initialize_consumer(consumer_name: &ConsumerType) -> Result<Box<Consumer>, String> {
    match consumer_name {
        ConsumerType::StdOut => {
            Ok(Box::new(StdOutConsumer{}))
        },
        ConsumerType::Influxdb => {
            Ok(Box::new(InfluxdbConsumer::new()))
        },
    }
}

pub struct StdOutConsumer;

impl Consumer for StdOutConsumer {
    fn consume(&self, _sensor: &Box<BTSensor>, measurement: &HashMap<String, Value>) {
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
    fn consume(&self, sensor: &Box<BTSensor>, measurement: &HashMap<String, Value>) {
        let mut point = Point::new(sensor.get_tag());
        point.add_tag(
            "address",
            InfluxVal::String(sensor.get_address().to_string())
        );

        for (key, val) in measurement {
            match val {
                Value::String(s) => {
                    point.add_field(key, InfluxVal::String(s.to_string()));
                },
                Value::Integer(i) => {
                    point.add_field(key, InfluxVal::Integer(*i));
                },
                Value::Float(f) => {
                    point.add_field(key, InfluxVal::Float(*f));
                },
                Value::Boolean(b) => {
                    point.add_field(key, InfluxVal::Boolean(*b));
                },
            }
        }
        match self.client.write_point(point, Some(Precision::Milliseconds), None) {
            Err(e) => {
                error!("{:?}", e);
            },
            _ => (),
        };
    }
}

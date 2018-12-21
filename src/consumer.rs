use std::collections::HashMap;

use serde_json;

use bt_sensor::Value;

pub trait Consumer {
    fn consume(&self, measurement: &HashMap<String, Value>);
}

pub fn initialize_consumer(consumer_name: &str) -> Result<Box<Consumer>, String> {
    match consumer_name {
        "stdout" => {
            Ok(Box::new(StdOutConsumer{}))
        },
        _ => {
            Err(format!("No consumer {}", consumer_name))
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

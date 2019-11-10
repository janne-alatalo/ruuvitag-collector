use std::env;

use influx_db_client::{
    Client, Point, Points, Value as InfluxVal, Precision
};

use bt_sensor::{Value, BTSensor};

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub enum ConsumerType {
    StdOut,
    StdOutJson,
    Influxdb,
}

pub trait Consumer {
    fn consume(&mut self, sensors: &[&dyn BTSensor]);
}

pub fn initialize_consumer(consumer_name: &ConsumerType) -> Result<Box<dyn Consumer>, String> {
    match consumer_name {
        ConsumerType::StdOut => {
            Ok(Box::new(StdOutConsumer{}))
        },
        ConsumerType::StdOutJson => {
            Ok(Box::new(StdOutJsonConsumer{}))
        },
        ConsumerType::Influxdb => {
            Ok(Box::new(InfluxdbConsumer::new()))
        },
    }
}

pub struct StdOutConsumer;

impl Consumer for StdOutConsumer {
    fn consume(&mut self, sensors: &[&dyn BTSensor]) {
        for sensor in sensors {
            if sensor.get_bt_device().is_upto_date() {
                match sensor.get_measurements_str() {
                    Some(s) => {
                        println!("Address: {}", sensor.get_address());
                        println!("{}", s);
                    },
                    None => (),
                }
            }
        }
    }
}

pub struct StdOutJsonConsumer;

impl Consumer for StdOutJsonConsumer {
    fn consume(&mut self, sensors: &[&dyn BTSensor]) {
        for sensor in sensors {
            if sensor.get_bt_device().is_upto_date() {
                match sensor.get_measurements_json_str() {
                    Some(s) => println!("{}", s),
                    None => (),
                }
            }
        }
    }
}

pub struct InfluxdbConsumer {
    client: Client,
    measurements: Vec<Point>,
}

impl InfluxdbConsumer {
    fn new() -> InfluxdbConsumer {
        let influx_url = env::var_os("INFLUXDB_URL")
            .map(|s| s.to_str().expect("INFLUXDB_URL conversion error").to_string())
            .unwrap_or("http://127.0.0.1:8086".into());
        let influx_db = env::var_os("INFLUXDB_DB")
            .map(|s| s.to_str().expect("INFLUXDB_DB conversion error").to_string())
            .unwrap_or("ruuvitag".into());
        let influx_user = env::var_os("INFLUXDB_USER")
            .map(|s| s.to_str().expect("INFLUXDB_USER conversion error").to_string())
            .unwrap_or("ruuvitag".into());
        let influx_password = env::var_os("INFLUXDB_PASSWORD")
            .map(|s| s.to_str().expect("INFLUXDB_USER conversion error").to_string())
            .unwrap_or("super_secret_ruuvitag_password".into());
        let mut client = Client::new(influx_url, influx_db)
            .set_authentication(influx_user, influx_password);
        client.set_write_timeout(3);
        InfluxdbConsumer{client, measurements: Vec::new()}
    }
}

impl Consumer for InfluxdbConsumer {
    fn consume(&mut self, sensors: &[&dyn BTSensor]) {
        let mut points_vec = Vec::<Point>::new();
        for sensor in sensors {
            if !sensor.get_bt_device().is_upto_date() {
                continue;
            }
            match sensor.get_measurements() {
                Some(measurements) => {
                    let mut point = Point::new("ruuvitag");
                    point.add_tag(
                        "tag",
                        InfluxVal::String(sensor.get_tag().to_string())
                    );
                    point.add_tag(
                        "address",
                        InfluxVal::String(sensor.get_address().to_string())
                    );
                    point.add_timestamp(sensor.get_measurement_timestamp() as i64);

                    for (key, val) in measurements {
                        match val {
                            Value::String(s) => {
                                point.add_field(key, InfluxVal::String(s.to_string()));
                            },
                            Value::Integer(i) => {
                                point.add_field(key, InfluxVal::Integer(i));
                            },
                            Value::Float(f) => {
                                point.add_field(key, InfluxVal::Float(f));
                            },
                            Value::Boolean(b) => {
                                point.add_field(key, InfluxVal::Boolean(b));
                            },
                        }
                    }
                    points_vec.push(point);
                },
                None => (),
            }
        }
        self.measurements.append(&mut points_vec);
        if self.measurements.len() > 0 {
            debug!("Writing {} points to influxdb", self.measurements.len());
            let points = Points::create_new(self.measurements.clone());
            match self.client.write_points(points, Some(Precision::Milliseconds), None) {
                Ok(()) => {
                    self.measurements = Vec::new();
                },
                Err(e) => {
                    error!("{:?}", e);
                },
            };
        }
    }
}

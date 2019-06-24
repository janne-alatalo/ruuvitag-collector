use std::collections::HashMap;
use std::time::{SystemTime, Duration};

use bt_sensor::BTSensor;
use discovery_mode::DiscoveryMode;

pub struct BTDevice {
    address: String,
    tag: String,
    object_path: String,
    mfr_data: Option<HashMap<u16, Vec<u8>>>,
    svc_data: Option<HashMap<String, Vec<u8>>>,
    measurement_timestamp: u64,
    last_seen: SystemTime,
    last_seen_forget: Duration,
    discovery_mode: DiscoveryMode,
    bt_sensor: Option<Box<BTSensor>>,
}

impl BTDevice {

    pub fn new(
        object_path: String,
        address: String,
        tag: String,
        mfr_data: Option<HashMap<u16, Vec<u8>>>,
        svc_data: Option<HashMap<String, Vec<u8>>>,
        measurement_timestamp: u64,
        last_seen_forget: Duration,
        discovery_mode: DiscoveryMode,
        ) -> BTDevice
    {

        BTDevice{
            address: address,
            tag: tag,
            object_path: object_path,
            mfr_data: mfr_data,
            svc_data: svc_data,
            measurement_timestamp,
            last_seen: SystemTime::now(),
            last_seen_forget,
            discovery_mode,
            bt_sensor: None,
        }

    }

    pub fn get_object_path(&self) -> &str {
        &self.object_path
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_mfr_data(&self) -> Option<&HashMap<u16, Vec<u8>>> {
        self.mfr_data.as_ref()
    }

    pub fn get_svc_data(&self) -> Option<&HashMap<String, Vec<u8>>> {
        self.svc_data.as_ref()
    }

    pub fn set_address(&mut self, address: String) {
        self.address = address;
    }

    pub fn update_data(
        &mut self,
        mfr_data: Option<HashMap<u16,Vec<u8>>>,
        svc_data: Option<HashMap<String, Vec<u8>>>,
        meas_timestamp: u64
        ) -> ()
    {
        if self.set_mfr_data(mfr_data) || self.set_svc_data(svc_data) {
            self.set_measurement_timestamp(meas_timestamp);
            self.reset_last_seen();
        }
    }

    pub fn set_mfr_data(&mut self, mfr_data: Option<HashMap<u16, Vec<u8>>>) -> bool {
        if self.mfr_data == mfr_data {
            debug!("{} mfr data not updated", self.get_address());
            return false
        }
        self.mfr_data = mfr_data;
        true
    }

    pub fn set_svc_data(&mut self, svc_data: Option<HashMap<String, Vec<u8>>>) -> bool {
        if self.svc_data == svc_data {
            debug!("{} svc data not updated", self.get_address());
            return false
        }
        self.svc_data = svc_data;
        true
    }

    pub fn set_measurement_timestamp(&mut self, meas_timestamp: u64) {
        self.measurement_timestamp = meas_timestamp;
    }

    pub fn get_measurement_timestamp(&self) -> u64 {
        self.measurement_timestamp
    }

    pub fn get_tag(&self) -> &str {
        &self.tag
    }

    pub fn reset_last_seen(&mut self) {
        self.last_seen = SystemTime::now();
    }

    pub fn seen_more_resently_than(&self, d: Duration) -> bool {
        let elapsed = match self.last_seen.elapsed() {
            Ok(e) => e,
            // In very rare case this could fail. In that case return dummy one millisecond
            // duration.
            Err(_) => Duration::from_millis(1),
        };
        d > elapsed
    }

    pub fn is_upto_date(&self) -> bool {
        self.seen_more_resently_than(self.last_seen_forget)
    }

    pub fn get_discovery_mode(&self) -> DiscoveryMode {
        self.discovery_mode.clone()
    }

    pub fn set_sensor(&mut self, sensor: Box<BTSensor>) {
        self.bt_sensor = Some(sensor);
    }

    pub fn unset_sensor(&mut self) {
        self.bt_sensor = None;
    }

    pub fn get_sensor(&self) -> Option<&BTSensor> {
        self.bt_sensor.as_ref().map(|b| &**b)
    }

}

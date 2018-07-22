use serde_json;

use bt_sensor::{DiscoveryMode, BTSensor, SensorIFConstr};
use bt_device::BTDevice;

#[derive(Default, Debug)]
pub struct RuuvitagDF2 {
    last_seen: i32,
    discovery_mode: DiscoveryMode,
    bt_device: BTDevice,
}

impl BTSensor for RuuvitagDF2 {

    fn is_valid_data(&self, device: &BTDevice) -> bool {
        RuuvitagDF2::_is_valid_mfr_data(device)
    }

    fn get_measurements_json_str(&self) -> Option<String> {
        self._get_measurements_json_str()
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

}

static SVC_DATA_UUID: &'static str = "0000feaa-0000-1000-8000-00805f9b34fb";

impl RuuvitagDF2 {

    pub fn new() -> RuuvitagDF2 {
        RuuvitagDF2{..Default::default()}
    }

    pub fn _is_valid_mfr_data(device: &BTDevice) -> bool {
        match device.get_svc_data().and_then(|m| m.get(SVC_DATA_UUID)) {
            Some(data) => {
                if data.len() == 7 {
                    return true;
                }
                false
            },
            None => {
                false
            }
        }
    }


    pub fn get_sensor_if_constructor() -> (&'static str, SensorIFConstr) {
        (
            "RuuvitagDF2",
            Box::new(|bt_device|
                 Box::new(RuuvitagDF2{bt_device: bt_device, ..Default::default()})
                 )
        )
    }

    // See https://github.com/ruuvi/ruuvi-sensor-protocols#data-format-3-protocol-specification
    // for the specification
    pub fn get_data_format(&self) -> Option<u8> {
        self.bt_device
            .get_svc_data()?
            .get(SVC_DATA_UUID)?
            .get(0)
            .map(|v| *v)
    }

    pub fn get_humidity(&self) -> Option<f32> {
        let humidity = self.bt_device
            .get_svc_data()?
            .get(SVC_DATA_UUID)?
            .get(1)
            .map(|v| *v)?;
        Some((humidity as f32) * 0.5_f32)
    }

    pub fn get_temp_wholes(&self) -> Option<i8> {
        self.bt_device
            .get_svc_data()?
            .get(SVC_DATA_UUID)?
            .get(2)
            .map(|u8_temp| {
                let i8_temp = (0x7F & u8_temp) as i8;
                match u8_temp & 0x80 {
                    0 => i8_temp,
                    _ => i8_temp * -1,
                }
            })
    }

    pub fn get_temp_fractions(&self) -> Option<u8> {
        self.bt_device
            .get_svc_data()?
            .get(SVC_DATA_UUID)?
            .get(3)
            .map(|v| *v)
    }

    pub fn get_pressure(&self) -> Option<u16> {
        let pressure_top = self.bt_device
            .get_svc_data()?
            .get(SVC_DATA_UUID)?
            .get(4)?;
        let pressure_bottom = self.bt_device
            .get_svc_data()?
            .get(SVC_DATA_UUID)?
            .get(5)?;
        Some(((*pressure_top as u16) << 8) | *pressure_bottom as u16)
    }


    pub fn get_id(&self) -> Option<u8> {
        self.bt_device
            .get_svc_data()?
            .get(SVC_DATA_UUID)?
            .get(6)
            .map(|v| *v)
    }

    pub fn get_status(&self) -> String {
        if let (
            Some(format), Some(hum), Some(temp_wholes), Some(temp_fract),
            Some(press), Some(id)) = (
            self.get_data_format(), self.get_humidity(), self.get_temp_wholes(),
            self.get_temp_fractions(), self.get_pressure(), self.get_id()) {

            let hum_perc = hum as f32 * 0.5;
            let press_corr = 50000 + press as u32;

            format!("format {}\ntemp {},{}℃\thumidity {:.1}%\tpressure {} Pa\nid {}\n",
                    format, temp_wholes, temp_fract, hum_perc, press_corr, id)
        }
        else {
            String::from("Invalid manufacturer data")
        }

    }

    fn _get_measurements_json_str(&self) -> Option<String> {

        if let (
            Some(format), Some(hum), Some(temp_wholes),
            Some(temp_fract), Some(press), Some(id)) = (
            self.get_data_format(), self.get_humidity(), self.get_temp_wholes(),
            self.get_temp_fractions(), self.get_pressure(), self.get_id()) {

            let press_corr = 50000 + press as u32;

            let tag = self.bt_device.get_tag().to_string();
            let address = self.bt_device.get_address().to_string();

            let meas = RuuvitagDF2Meas{
                data_format: format,
                humidity: hum,
                temperature: temp_wholes,
                temperature_fractions: temp_fract,
                pressure: press_corr,
                id: id,
                address: address,
                tag: tag,
            };
            serde_json::to_string(&meas).ok()

        } else {
            None
        }

    }

}

#[derive(Default, Debug, Serialize)]
pub struct RuuvitagDF2Meas {
    data_format: u8,
    humidity: f32,
    temperature: i8,
    temperature_fractions: u8,
    pressure: u32,
    id: u8,
    address: String,
    tag: String,
}

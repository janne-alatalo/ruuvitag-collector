use std::collections::HashMap;

use serde_json;

use bt_sensor::{DiscoveryMode, BTSensor, SensorIFConstr};
use bt_device::BTDevice;

#[derive(Default, Debug)]
pub struct RuuvitagDF3 {
    last_seen: i32,
    discovery_mode: DiscoveryMode,
    bt_device: BTDevice,
}

impl BTSensor for RuuvitagDF3 {

    fn is_valid_data(&self, device: &BTDevice) -> bool {
        RuuvitagDF3::_is_valid_mfr_data(device)
    }

    fn get_measurements(&self) -> Option<HashMap<String, i32>> {
        self._get_measurements()
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

static MFR_DATA_FIELD: u16 = 0x0499;

impl RuuvitagDF3 {

    pub fn new() -> RuuvitagDF3 {
        RuuvitagDF3{..Default::default()}
    }

    pub fn _is_valid_mfr_data(device: &BTDevice) -> bool {
        match device.get_mfr_data().get(&MFR_DATA_FIELD) {
            Some(data) => {
                if data.len() == 14 {
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
            "RuuvitagDF3",
            Box::new(|bt_device|
                 Box::new(RuuvitagDF3{bt_device: bt_device, ..Default::default()})
                 )
        )
    }

    // See https://github.com/ruuvi/ruuvi-sensor-protocols#data-format-3-protocol-specification
    // for the specification
    pub fn get_data_format(&self) -> Option<u8> {
        let format = self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(0)?;
        Some(*format)
    }

    pub fn get_humidity(&self) -> Option<u8> {
        let humidity = self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(1)?;
        Some(*humidity)
    }

    pub fn get_temp_wholes(&self) -> Option<i8> {
        let u8_temp = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(2)?;
        let i8_temp = (0x7F & u8_temp) as i8;
        if (u8_temp & 0x80) == 0 {
            return Some(i8_temp)
        }
        Some(i8_temp * -1)
    }

    pub fn get_temp_fractions(&self) -> Option<u8> {
        let temp = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(3)?;
        Some(temp)
    }

    pub fn get_pressure(&self) -> Option<u16> {
        let pressure_top = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(4)?;
        let pressure_bottom = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(5)?;
        let pressure = ((pressure_top as u16) << 8) | pressure_bottom as u16;
        Some(pressure)
    }

    pub fn get_acceleration_x(&self) -> Option<i16> {
        let acceleration_x_top = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(6)?;
        let acceleration_x_bottom = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(7)?;
        let acceleration_x = ((acceleration_x_top as u16) << 8) | acceleration_x_bottom as u16;
        Some(acceleration_x as i16)
    }

    pub fn get_acceleration_y(&self) -> Option<i16> {
        let acceleration_y_top = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(8)?;
        let acceleration_y_bottom = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(9)?;
        let acceleration_y = ((acceleration_y_top as u16) << 8) | acceleration_y_bottom as u16;
        Some(acceleration_y as i16)
    }

    pub fn get_acceleration_z(&self) -> Option<i16> {
        let acceleration_z_top = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(10)?;
        let acceleration_z_bottom = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(11)?;
        let acceleration_z = ((acceleration_z_top as u16) << 8) | acceleration_z_bottom as u16;
        Some(acceleration_z as i16)
    }

    pub fn get_battery(&self) -> Option<u16> {
        let battery_top = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(10)?;
        let battery_bottom = *self.bt_device.get_mfr_data().get(&MFR_DATA_FIELD)?.get(11)?;
        let battery = ((battery_top as u16) << 8) | battery_bottom as u16;
        Some(battery)
    }

    pub fn get_status(&self) -> String {
        if let (
            Some(format), Some(hum), Some(temp_wholes),
            Some(temp_fract), Some(press), Some(acc_x),
            Some(acc_y), Some(acc_z), Some(batt)) = (
            self.get_data_format(), self.get_humidity(), self.get_temp_wholes(),
            self.get_temp_fractions(), self.get_pressure(), self.get_acceleration_x(),
            self.get_acceleration_y(), self.get_acceleration_z(), self.get_battery()) {

            let hum_perc = hum as f32 * 0.5;
            let press_corr = 50000 + press as u32;

            format!("format {}\tbattery {}\ntemp {},{}℃\thumidity {:.1}%\tpressure {} Pa\nacc-x {}\tacc-y {}\tacc-z {}",
                    format, batt, temp_wholes, temp_fract, hum_perc, press_corr, acc_x, acc_y, acc_z)
        }
        else {
            String::from("Invalid manufacturer data")
        }

    }

    fn _get_measurements(&self) -> Option<HashMap<String, i32>> {

        if let (
            Some(format), Some(hum), Some(temp_wholes),
            Some(temp_fract), Some(press), Some(acc_x),
            Some(acc_y), Some(acc_z), Some(batt)) = (
            self.get_data_format(), self.get_humidity(), self.get_temp_wholes(),
            self.get_temp_fractions(), self.get_pressure(), self.get_acceleration_x(),
            self.get_acceleration_y(), self.get_acceleration_z(), self.get_battery()) {

            let press_corr = 50000 + press as u32;

            let mut map = HashMap::new();
            map.insert("format".to_string(), format as i32);
            map.insert("battery".to_string(), batt as i32);
            map.insert("temp".to_string(), temp_wholes as i32);
            map.insert("temp_fractions".to_string(), temp_fract as i32);
            map.insert("humidity".to_string(), hum as i32);
            map.insert("pressure".to_string(), press_corr as i32);
            map.insert("acc_x".to_string(), acc_x as i32);
            map.insert("acc_y".to_string(), acc_y as i32);
            map.insert("acc_z".to_string(), acc_z as i32);
            Some(map)

        } else {
            None
        }
    }

    fn _get_measurements_json_str(&self) -> Option<String> {

        if let (
            Some(format), Some(hum), Some(temp_wholes),
            Some(temp_fract), Some(press), Some(acc_x),
            Some(acc_y), Some(acc_z), Some(batt)) = (
            self.get_data_format(), self.get_humidity(), self.get_temp_wholes(),
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
                temperature_fractions: temp_fract,
                pressure: press_corr,
                acceleration_x: acc_x,
                acceleration_y: acc_y,
                acceleration_z: acc_z,
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
pub struct RuuvitagDF3Meas {
    data_format: u8,
    battery: u16,
    humidity: u8,
    temperature: i8,
    temperature_fractions: u8,
    pressure: u32,
    acceleration_x: i16,
    acceleration_y: i16,
    acceleration_z: i16,
    address: String,
    tag: String,
}

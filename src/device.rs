use std::collections::HashMap;

static MFR_DATA_FIELD: u16 = 0x0499;

#[derive(Default, Debug)]
pub struct Device {
    object_path: String,
    address: Option<String>,
    mfr_data: Option<HashMap<u16, Vec<u8>>>,
}

impl Device {

    pub fn new(object_path: String) -> Device {
        Device{object_path, ..Default::default()}
    }

    pub fn set_address(&mut self, address: String) -> &mut Self {
        self.address = Some(address);
        self
    }

    pub fn set_mfr_data(&mut self, mfr_data: HashMap<u16, Vec<u8>>) -> &mut Self {
        self.mfr_data = Some(mfr_data);
        self
    }

    // See https://github.com/ruuvi/ruuvi-sensor-protocols#data-format-3-protocol-specification
    // for the specification
    pub fn get_data_format(&self) -> Option<u8> {
        match self.mfr_data {
            Some(ref mfr_data) => {
                let format = mfr_data.get(&MFR_DATA_FIELD)?.get(0)?;
                Some(*format)
            },
            None => None,
        }
    }

    pub fn get_humidity(&self) -> Option<u8> {
        match self.mfr_data {
            Some(ref mfr_data) => {
                let humidity = mfr_data.get(&MFR_DATA_FIELD)?.get(1)?;
                Some(*humidity)
            },
            None => None,
        }
    }

    pub fn get_temp_wholes(&self) -> Option<i8> {
        match self.mfr_data {
            Some(ref mfr_data) => {
                let u8_temp = *mfr_data.get(&MFR_DATA_FIELD)?.get(2)?;
                let i8_temp = (0x7F & u8_temp) as i8;
                if (u8_temp & 0x80) == 0 {
                    return Some(i8_temp)
                }
                Some(i8_temp * -1)
            },
            None => None,
        }
    }

    pub fn get_temp_fractions(&self) -> Option<u8> {
        match self.mfr_data {
            Some(ref mfr_data) => {
                let temp = *mfr_data.get(&MFR_DATA_FIELD)?.get(3)?;
                Some(temp)
            },
            None => None,
        }
    }

    pub fn get_pressure(&self) -> Option<u16> {
        match self.mfr_data {
            Some(ref mfr_data) => {
                let pressure_top = *mfr_data.get(&MFR_DATA_FIELD)?.get(4)?;
                let pressure_bottom = *mfr_data.get(&MFR_DATA_FIELD)?.get(5)?;
                let pressure = ((pressure_top as u16) << 8) | pressure_bottom as u16;
                Some(pressure)
            },
            None => None,
        }
    }

    pub fn get_acceleration_x(&self) -> Option<u16> {
        match self.mfr_data {
            Some(ref mfr_data) => {
                let acceleration_x_top = *mfr_data.get(&MFR_DATA_FIELD)?.get(6)?;
                let acceleration_x_bottom = *mfr_data.get(&MFR_DATA_FIELD)?.get(7)?;
                let acceleration_x = ((acceleration_x_top as u16) << 8) | acceleration_x_bottom as u16;
                Some(acceleration_x)
            },
            None => None,
        }
    }

    pub fn get_acceleration_y(&self) -> Option<u16> {
        match self.mfr_data {
            Some(ref mfr_data) => {
                let acceleration_y_top = *mfr_data.get(&MFR_DATA_FIELD)?.get(8)?;
                let acceleration_y_bottom = *mfr_data.get(&MFR_DATA_FIELD)?.get(9)?;
                let acceleration_y = ((acceleration_y_top as u16) << 8) | acceleration_y_bottom as u16;
                Some(acceleration_y)
            },
            None => None,
        }
    }

    pub fn get_acceleration_z(&self) -> Option<u16> {
        match self.mfr_data {
            Some(ref mfr_data) => {
                let acceleration_z_top = *mfr_data.get(&MFR_DATA_FIELD)?.get(10)?;
                let acceleration_z_bottom = *mfr_data.get(&MFR_DATA_FIELD)?.get(11)?;
                let acceleration_z = ((acceleration_z_top as u16) << 8) | acceleration_z_bottom as u16;
                Some(acceleration_z)
            },
            None => None,
        }
    }

    pub fn get_battery(&self) -> Option<u16> {
        match self.mfr_data {
            Some(ref mfr_data) => {
                let battery_top = *mfr_data.get(&MFR_DATA_FIELD)?.get(10)?;
                let battery_bottom = *mfr_data.get(&MFR_DATA_FIELD)?.get(11)?;
                let battery = ((battery_top as u16) << 8) | battery_bottom as u16;
                Some(battery)
            },
            None => None,
        }
    }

    pub fn get_status(&self) -> String {
        if let (
            Some(format), Some(hum), Some(temp_wholes),
            Some(temp_fract), Some(press), Some(acc_x),
            Some(acc_y), Some(acc_z), Some(batt)) = (
            self.get_data_format(), self.get_humidity(), self.get_temp_wholes(),
            self.get_temp_fractions(), self.get_pressure(), self.get_acceleration_x(),
            self.get_acceleration_y(), self.get_acceleration_z(), self.get_battery()) {

            format!("format {}\tbattery {}\ntemp {},{}℃\thumidity {}\tpressure {}\nacc-x {}\tacc-y {}\tacc-z {}",
                    format, batt, temp_wholes, temp_fract, hum, press, acc_x, acc_y, acc_z)
        }
        else {
            String::from("Invalid manufacturer data")
        }

    }

}

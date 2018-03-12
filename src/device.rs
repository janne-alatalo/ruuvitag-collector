use std::collections::HashMap;

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

}

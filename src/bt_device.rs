use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct BTDevice {
    address: String,
    tag: String,
    object_path: String,
    mfr_data: Option<HashMap<u16, Vec<u8>>>,
    svc_data: Option<HashMap<String, Vec<u8>>>,
}

impl BTDevice {

    pub fn new(
        object_path: String,
        address: String,
        tag: String,
        mfr_data: Option<HashMap<u16, Vec<u8>>>,
        svc_data: Option<HashMap<String, Vec<u8>>>,
        ) -> BTDevice
    {

        BTDevice{
            address: address,
            tag: tag,
            object_path: object_path,
            mfr_data: mfr_data,
            svc_data: svc_data,
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

    pub fn set_mfr_data(&mut self, mfr_data: Option<HashMap<u16, Vec<u8>>>) {
        self.mfr_data = mfr_data;
    }

    pub fn set_svc_data(&mut self, svc_data: Option<HashMap<String, Vec<u8>>>) {
        self.svc_data = svc_data;
    }

    pub fn get_tag(&self) -> &str {
        &self.tag
    }

}

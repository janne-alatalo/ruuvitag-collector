use std::collections::HashMap;

#[derive(Default)]
pub struct SensorConf {
    auto: bool,
    address_map: HashMap<String, String>,
}

impl SensorConf {

    pub fn new() -> SensorConf {
        SensorConf{
            auto: true,
            address_map: HashMap::new(),
        }
    }

    pub fn is_auto(&self) -> bool {
        self.auto
    }

    pub fn get_address_map(&self) -> &HashMap<String, String> {
        &self.address_map
    }

}

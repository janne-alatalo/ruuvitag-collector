#[derive(Debug, Clone)]
pub enum DiscoveryMode {
    Auto,
    Configured(String),
}

impl Default for DiscoveryMode {
    fn default() -> DiscoveryMode { DiscoveryMode::Auto }
}


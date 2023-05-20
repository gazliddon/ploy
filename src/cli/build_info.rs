pub struct BuildInfo {
    pub version: String,
    pub authors: String,
    pub bin_name: String,
    pub crate_name: String,
}

impl Default for BuildInfo {
    fn default() -> Self {
        const UNKNOWN: &str = "UNKNOWN";
        let version = option_env!("CARGO_PKG_VERSION")
            .unwrap_or(UNKNOWN)
            .to_string();
        let authors = option_env!("CARGO_PKG_AUTHORS")
            .unwrap_or(UNKNOWN)
            .to_string();
        let bin_name = option_env!("CARGO_BIN_NAME").unwrap_or(UNKNOWN).to_string();
        let crate_name = option_env!("CARGO_CRATE_NAME")
            .unwrap_or(UNKNOWN)
            .to_string();

        Self {
            version,
            authors,
            bin_name,
            crate_name,
        }
    }
}

impl BuildInfo {
    pub fn new() -> Self {
        Default::default()
    }
}

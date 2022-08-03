use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct FilterCfg {
    pub mmdb_url: String,
    pub mmdb_cluster: String,
}

impl FilterCfg {
    pub fn new(bytes: Vec<u8>) -> Result<FilterCfg, serde_json::Error> {
        serde_json::from_slice(bytes.as_slice())
    }
}
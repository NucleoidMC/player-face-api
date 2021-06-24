use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub requests_per_minute: u32,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            requests_per_minute: 100,
            port: 1111,
        }
    }
}

pub(super) fn load() -> Config {
    let path = Path::new("config.json");
    if path.exists() {
        let mut file = File::open(path).unwrap();
        serde_json::from_reader(&mut file).unwrap()
    } else {
        let config = Config::default();

        let mut file = File::create(path).unwrap();
        serde_json::to_writer_pretty(&mut file, &config).unwrap();

        config
    }
}

use crate::{Continent, Mirror};

use serde::Deserialize;
use std::collections::HashMap;
use std::num::NonZeroU16;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ConfigError {
    #[error(r#"continents must contain "default""#)]
    NoDefaultContinent,
    #[error(r#"continent {continent:?} mention unknown mirror {mirror}"#)]
    MirrorUnknown {
        continent: Continent,
        mirror: String,
    },
    #[error(r#"continent {0} is not supported, connect Earth goverment to fix it"#)]
    ContinentUnknown(String),
    #[error(r#"no mirrors are specified"#)]
    NoMirrors,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_host")]
    pub host: String,
    #[serde(default = "Config::default_ip_headers")]
    pub ip_headers: Vec<String>,
    #[serde(default = "Config::default_ip_headers_recursive")]
    pub ip_headers_recursive: bool,
    #[serde(default = "Config::default_healthcheck_interval")]
    pub healthckeck_interval: NonZeroU16,
    #[serde(default = "Config::default_log_level")]
    pub log_level: log::Level,
    #[serde(default)]
    pub response_headers: HashMap<String, String>,
    pub geolite2: String,
    pub mirrors: HashMap<String, Mirror>,
    pub continents: HashMap<String, Vec<String>>,
}

impl Config {
    fn default_host() -> String {
        "127.0.0.1:8080".into()
    }

    fn default_ip_headers() -> Vec<String> {
        vec!["X-FORWARDED-FOR".into()]
    }

    fn default_ip_headers_recursive() -> bool {
        true
    }

    fn default_healthcheck_interval() -> NonZeroU16 {
        5.try_into().unwrap()
    }

    fn default_log_level() -> log::Level {
        log::Level::Info
    }
}

pub fn parse_config<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
    let toml_string = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&toml_string)?;
    Ok(config)
}

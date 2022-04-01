use crate::{Continent, Mirror};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(r#"continents must contain "default""#)]
    NoDefaultContinent,
    #[error(r#"continent {continent:?} mention unknown mirror {mirror}"#)]
    UnknwonMirror {
        continent: Continent,
        mirror: String,
    },
    #[error(r#"continent {0} is not supported, connect Earth goverment to fix it"#)]
    ContinentUnknown(String),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub geolite2: String,
    pub mirrors: HashMap<String, Mirror>,
    pub continents: HashMap<String, Vec<String>>,
}

pub fn parse_config<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
    let toml_string = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&toml_string)?;
    Ok(config)
}

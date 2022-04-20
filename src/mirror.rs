use crate::config::{Config, ConfigError};
use crate::geo::Continent;
use serde::Deserialize;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "MirrorConfig")]
pub struct Mirror {
    pub upstream: Url,
    pub healthcheck: Url,
    pub available: Arc<AtomicBool>,
}

#[derive(Debug, Deserialize)]
struct MirrorConfig {
    upstream: String,
    healthcheck: String,
}

impl TryFrom<MirrorConfig> for Mirror {
    type Error = url::ParseError;

    fn try_from(value: MirrorConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            upstream: value.upstream.as_str().try_into()?,
            healthcheck: value.healthcheck.as_str().try_into()?,
            available: Arc::new(AtomicBool::new(false)),
        })
    }
}

pub type MirrorVec = Arc<SmallVec<[Mirror; 4]>>;

#[derive(Debug, Clone)]
pub struct ContinentMap {
    map: HashMap<Continent, MirrorVec>,
    mirrors: Vec<Mirror>,
}

impl ContinentMap {
    pub fn from_config(config: &Config) -> Result<Self, ConfigError> {
        let Config {
            mirrors: conf_mirrors,
            continents: conf_continents,
            ..
        } = config;

        conf_continents
            .get("default")
            .ok_or(ConfigError::NoDefaultContinent)?;

        if conf_mirrors.is_empty() {
            return Err(ConfigError::NoMirrors);
        }

        Ok(Self {
            mirrors: conf_mirrors.values().cloned().collect(),
            map: conf_continents
                .iter()
                .map(|(continent_string, mirror_strings)| {
                    let continent = continent_string
                        .as_str()
                        .try_into()
                        .map_err(|_| ConfigError::ContinentUnknown(continent_string.to_owned()))?;
                    let mirrors = Arc::new(
                        mirror_strings
                            .iter()
                            .map(|s| {
                                conf_mirrors
                                    .get(s)
                                    .ok_or_else(|| ConfigError::MirrorUnknown {
                                        continent,
                                        mirror: s.to_owned(),
                                    })
                                    .cloned()
                            })
                            .collect::<Result<_, ConfigError>>()?,
                    );
                    Ok((continent, mirrors))
                })
                .collect::<Result<HashMap<Continent, MirrorVec>, ConfigError>>()?,
        })
    }

    pub fn get(&self, continent: Continent) -> MirrorVec {
        self.map
            .get(&continent)
            .cloned()
            .unwrap_or_else(|| self.get_default())
    }

    pub fn get_default(&self) -> MirrorVec {
        self.map.get(&Continent::Default).unwrap().clone()
    }

    pub fn all_mirrors(&self) -> &[Mirror] {
        &self.mirrors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config() {
        let s = r#"
        geolite2 = ""
        
        [mirrors]
        
        [continents]
        "#;
        let config: Config = toml::from_str(s).unwrap();
        assert!(ContinentMap::from_config(&config).is_err());
    }

    #[test]
    fn empty_continents() {
        let s = r#"
        geolite2 = ""
        
        [mirrors]
        mirror = { upstream = "http://example.com", healthcheck = "http://example.com/ping" }
        
        [continents]
        "#;
        let config: Config = toml::from_str(s).unwrap();
        assert_eq!(
            ContinentMap::from_config(&config).unwrap_err(),
            ConfigError::NoDefaultContinent
        );
    }

    #[test]
    fn empty_mirrors() {
        let s = r#"
        geolite2 = ""
        
        [mirrors]
        
        [continents]
        default = []
        "#;
        let config: Config = toml::from_str(s).unwrap();
        assert_eq!(
            ContinentMap::from_config(&config).unwrap_err(),
            ConfigError::NoMirrors
        );
    }

    #[test]
    fn no_default_continent() {
        let s = r#"
        geolite2 = ""
        
        [mirrors]
        mirror = { upstream = "http://example.com", healthcheck = "http://example.com/ping" }
        
        [continents]
        Europe = ["mirror"]
        "#;
        let config: Config = toml::from_str(s).unwrap();
        assert_eq!(
            ContinentMap::from_config(&config).unwrap_err(),
            ConfigError::NoDefaultContinent
        );
    }

    #[test]
    fn wrong_mirror() {
        let s = r#"
        geolite2 = ""
        
        [mirrors]
        mirror1 = { upstream = "http://example.com", healthcheck = "http://example.com/ping" }
        
        [continents]
        default = ["mirror2"]
        "#;
        let config: Config = toml::from_str(s).unwrap();
        assert!(matches!(
            ContinentMap::from_config(&config).unwrap_err(),
            ConfigError::MirrorUnknown { .. }
        ));
    }

    #[test]
    fn wrong_continent_name() {
        let s = r#"
        geolite2 = ""
        
        [mirrors]
        mirror = { upstream = "http://example.com", healthcheck = "http://example.com/ping" }
        
        [continents]
        default = ["mirror"]
        Zeus = ["mirror"]
        "#;
        let config: Config = toml::from_str(s).unwrap();
        assert!(matches!(
            ContinentMap::from_config(&config).unwrap_err(),
            ConfigError::ContinentUnknown { .. }
        ));
    }
}

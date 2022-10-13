use crate::geo::Continent;

use hyper::http::uri::{InvalidUri, Uri};
use serde::Deserialize;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "MirrorConfig")]
pub struct Mirror {
    pub upstream: Uri,
    pub healthcheck: Uri,
    pub available: Arc<AtomicBool>,
}

#[derive(Debug, Deserialize)]
struct MirrorConfig {
    upstream: String,
    healthcheck: String,
}

impl TryFrom<MirrorConfig> for Mirror {
    type Error = InvalidUri;

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
    pub fn from_mirrors_and_continents(
        mirrors: &HashMap<String, Mirror>,
        continents: &HashMap<String, Vec<String>>,
    ) -> Result<Self, ContinentMapConfigError> {
        continents
            .get("default")
            .ok_or(ContinentMapConfigError::NoDefaultContinent)?;

        if mirrors.is_empty() {
            return Err(ContinentMapConfigError::NoMirrors);
        }

        Ok(Self {
            mirrors: mirrors.values().cloned().collect(),
            map: continents
                .iter()
                .map(|(continent_string, mirror_strings)| {
                    let continent = continent_string.as_str().try_into().map_err(|_| {
                        ContinentMapConfigError::ContinentUnknown(continent_string.to_owned())
                    })?;
                    let mirrors = Arc::new(
                        mirror_strings
                            .iter()
                            .map(|s| {
                                mirrors
                                    .get(s)
                                    .ok_or_else(|| ContinentMapConfigError::MirrorUnknown {
                                        continent,
                                        mirror: s.to_owned(),
                                    })
                                    .cloned()
                            })
                            .collect::<Result<_, ContinentMapConfigError>>()?,
                    );
                    Ok((continent, mirrors))
                })
                .collect::<Result<HashMap<Continent, MirrorVec>, ContinentMapConfigError>>()?,
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

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ContinentMapConfigError {
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::Config;

    pub fn continent_map_from_config(
        config: &Config,
    ) -> Result<ContinentMap, ContinentMapConfigError> {
        let Config {
            mirrors,
            continents,
            ..
        } = config;
        ContinentMap::from_mirrors_and_continents(mirrors, continents)
    }

    #[test]
    fn empty_config() {
        let s = r#"
        geolite2 = ""
        
        [mirrors]
        
        [continents]
        "#;
        let config: Config = toml::from_str(s).unwrap();
        assert!(continent_map_from_config(&config).is_err());
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
            continent_map_from_config(&config).unwrap_err(),
            ContinentMapConfigError::NoDefaultContinent
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
            continent_map_from_config(&config).unwrap_err(),
            ContinentMapConfigError::NoMirrors
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
            continent_map_from_config(&config).unwrap_err(),
            ContinentMapConfigError::NoDefaultContinent
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
            continent_map_from_config(&config).unwrap_err(),
            ContinentMapConfigError::MirrorUnknown { .. }
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
            continent_map_from_config(&config).unwrap_err(),
            ContinentMapConfigError::ContinentUnknown { .. }
        ));
    }
}

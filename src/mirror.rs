use crate::geo::Continent;

use hyper::http::uri::{InvalidUri, Uri};
use serde::Deserialize;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(try_from = "MirrorConfig")]
pub struct MirrorImpl {
    pub upstream: Uri,
    pub healthcheck: Uri,
    pub available: AtomicBool,
}

#[derive(Debug, Deserialize)]
struct MirrorConfig {
    upstream: String,
    healthcheck: String,
}

impl TryFrom<MirrorConfig> for MirrorImpl {
    type Error = InvalidUri;

    fn try_from(value: MirrorConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            upstream: value.upstream.as_str().try_into()?,
            healthcheck: value.healthcheck.as_str().try_into()?,
            available: AtomicBool::new(false),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "MirrorImpl")]
pub struct Mirror(Arc<MirrorImpl>);

impl AsRef<MirrorImpl> for Mirror {
    fn as_ref(&self) -> &MirrorImpl {
        &self.0
    }
}

impl Deref for Mirror {
    type Target = MirrorImpl;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MirrorImpl> for Mirror {
    fn from(value: MirrorImpl) -> Self {
        Self(Arc::new(value))
    }
}

pub type MirrorVec = SmallVec<[Mirror; 4]>;

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
                    let mirrors = mirror_strings
                        .iter()
                        .map(|s| {
                            mirrors
                                .get(s)
                                .ok_or_else(|| ContinentMapConfigError::MirrorUnknown {
                                    continent,
                                    mirror: s.to_string(),
                                })
                                .cloned()
                        })
                        .collect::<Result<_, ContinentMapConfigError>>()?;
                    Ok((continent, mirrors))
                })
                .collect::<Result<HashMap<Continent, MirrorVec>, ContinentMapConfigError>>()?,
        })
    }

    pub fn get(&self, continent: Continent) -> &MirrorVec {
        self.map
            .get(&continent)
            .unwrap_or_else(|| self.get_default())
    }

    pub fn get_default(&self) -> &MirrorVec {
        self.map.get(&Continent::Default).unwrap()
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

    #[derive(Debug, Deserialize)]
    struct MirrorsContinentsConfig {
        mirrors: HashMap<String, Mirror>,
        continents: HashMap<String, Vec<String>>,
    }

    fn continent_map_from_config(
        config: &MirrorsContinentsConfig,
    ) -> Result<ContinentMap, ContinentMapConfigError> {
        let MirrorsContinentsConfig {
            mirrors,
            continents,
            ..
        } = config;
        ContinentMap::from_mirrors_and_continents(mirrors, continents)
    }

    #[test]
    fn empty_config() {
        let s = r#"
        [mirrors]
        
        [continents]
        "#;
        let config: MirrorsContinentsConfig = toml::from_str(s).unwrap();
        assert!(continent_map_from_config(&config).is_err());
    }

    #[test]
    fn empty_continents() {
        let s = r#"
        [mirrors]
        mirror = { upstream = "http://example.com", healthcheck = "http://example.com/ping" }
        
        [continents]
        "#;
        let config: MirrorsContinentsConfig = toml::from_str(s).unwrap();
        assert_eq!(
            continent_map_from_config(&config).unwrap_err(),
            ContinentMapConfigError::NoDefaultContinent
        );
    }

    #[test]
    fn empty_mirrors() {
        let s = r#"
        [mirrors]
        
        [continents]
        default = []
        "#;
        let config: MirrorsContinentsConfig = toml::from_str(s).unwrap();
        assert_eq!(
            continent_map_from_config(&config).unwrap_err(),
            ContinentMapConfigError::NoMirrors
        );
    }

    #[test]
    fn no_default_continent() {
        let s = r#"
        [mirrors]
        mirror = { upstream = "http://example.com", healthcheck = "http://example.com/ping" }
        
        [continents]
        Europe = ["mirror"]
        "#;
        let config: MirrorsContinentsConfig = toml::from_str(s).unwrap();
        assert_eq!(
            continent_map_from_config(&config).unwrap_err(),
            ContinentMapConfigError::NoDefaultContinent
        );
    }

    #[test]
    fn wrong_mirror() {
        let s = r#"
        [mirrors]
        mirror1 = { upstream = "http://example.com", healthcheck = "http://example.com/ping" }
        
        [continents]
        default = ["mirror2"]
        "#;
        let config: MirrorsContinentsConfig = toml::from_str(s).unwrap();
        assert!(matches!(
            continent_map_from_config(&config).unwrap_err(),
            ContinentMapConfigError::MirrorUnknown { .. }
        ));
    }

    #[test]
    fn wrong_continent_name() {
        let s = r#"
        [mirrors]
        mirror = { upstream = "http://example.com", healthcheck = "http://example.com/ping" }
        
        [continents]
        default = ["mirror"]
        Zeus = ["mirror"]
        "#;
        let config: MirrorsContinentsConfig = toml::from_str(s).unwrap();
        assert!(matches!(
            continent_map_from_config(&config).unwrap_err(),
            ContinentMapConfigError::ContinentUnknown { .. }
        ));
    }
}

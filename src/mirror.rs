use crate::config::{Config, ConfigError};
use crate::geo::Continent;
use serde::Deserialize;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "MirrorConfig")]
pub struct Mirror {
    pub upstream: Url,
    pub healthcheck: Url,
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
        })
    }
}

pub type MirrorVec = Arc<SmallVec<[Mirror; 2]>>;

#[derive(Debug, Clone)]
pub struct ContinentMap(HashMap<Continent, MirrorVec>);

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
        Ok(Self(
            conf_continents
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
                                    .ok_or_else(|| ConfigError::UnknwonMirror {
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
        ))
    }

    pub fn get(&self, continent: Continent) -> MirrorVec {
        self.0
            .get(&continent)
            .cloned()
            .unwrap_or_else(|| self.get_default())
    }

    pub fn get_default(&self) -> MirrorVec {
        self.0.get(&Continent::Default).unwrap().clone()
    }
}

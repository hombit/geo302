pub use continent::Continent;
pub use error::GeoError;
use max_mind_db::MaxMindDbGeo;
use ripe_geo::{RipeGeo, RipeGeoOverlapsStrategy};

mod continent;
mod error;
pub mod max_mind_db;
pub mod ripe_geo;

use enum_dispatch::enum_dispatch;
use serde::Deserialize;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(try_from = "GeoConfig")]
#[enum_dispatch]
pub enum Geo {
    MaxMindDb(MaxMindDbGeo),
    RipeGeo(RipeGeo),
}

#[enum_dispatch(Geo)]
pub trait GeoTrait: Send + Sync {
    fn try_lookup_continent(&self, address: IpAddr) -> Result<Continent, GeoError>;
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum GeoConfig {
    #[serde(
        alias = "maxminddb",
        alias = "maxmind",
        alias = "MaxMind",
        alias = "Max Mind"
    )]
    MaxMindDb { path: PathBuf },
    #[serde(alias = "ripe-geo", alias = "ripegeo", alias = "ripe geo")]
    RipeGeo {
        path: PathBuf,
        overlaps: RipeGeoOverlapsStrategy,
    },
}

impl TryFrom<GeoConfig> for Geo {
    type Error = GeoError;

    fn try_from(value: GeoConfig) -> Result<Self, Self::Error> {
        let slf = match value {
            GeoConfig::MaxMindDb { path } => Self::MaxMindDb(MaxMindDbGeo::from_file(&path)?),
            GeoConfig::RipeGeo { path, overlaps } => {
                Self::RipeGeo(RipeGeo::from_folder(&path, overlaps)?)
            }
        };
        Ok(slf)
    }
}

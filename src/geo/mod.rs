pub use continent::Continent;
pub use error::GeoError;
#[cfg(feature = "ripe-geo")]
use ripe_geo::{RipeGeo, RipeGeoImpl, RipeGeoOverlapsStrategy};

mod continent;
mod error;
#[cfg(feature = "maxminddb")]
pub mod max_mind_db;
#[cfg(feature = "ripe-geo")]
pub mod ripe_geo;

use enum_dispatch::enum_dispatch;
use serde::Deserialize;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(try_from = "GeoConfig")]
#[enum_dispatch]
pub enum Geo {
    #[cfg(feature = "maxminddb")]
    MaxMindDb(max_mind_db::MaxMindDbGeo),
    #[cfg(feature = "ripe-geo")]
    RipeGeo(RipeGeo),
}

impl From<RipeGeoImpl> for Geo {
    fn from(value: RipeGeoImpl) -> Self {
        Geo::RipeGeo(value.into())
    }
}

#[enum_dispatch(Geo)]
pub trait GeoTrait: Send + Sync {
    fn try_lookup_continent(&self, address: IpAddr) -> Result<Continent, GeoError>;
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum GeoConfig {
    #[cfg(feature = "maxminddb")]
    #[serde(
        alias = "maxminddb",
        alias = "maxmind",
        alias = "MaxMind",
        alias = "Max Mind"
    )]
    MaxMindDb { path: PathBuf },
    #[cfg(feature = "ripe-geo")]
    #[serde(alias = "ripe-geo", alias = "ripegeo", alias = "ripe geo")]
    RipeGeo {
        #[cfg(feature = "ripe-geo-embedded")]
        #[serde(default)]
        path: Option<PathBuf>,
        #[cfg(not(feature = "ripe-geo-embedded"))]
        path: PathBuf,
        overlaps: Option<RipeGeoOverlapsStrategy>,
    },
}

impl TryFrom<GeoConfig> for Geo {
    type Error = GeoError;

    fn try_from(value: GeoConfig) -> Result<Self, Self::Error> {
        let slf = match value {
            #[cfg(feature = "maxminddb")]
            GeoConfig::MaxMindDb { path } => {
                Self::MaxMindDb(max_mind_db::MaxMindDbGeo::from_file(&path)?)
            }
            #[cfg(all(feature = "ripe-geo", not(feature = "ripe-geo-embedded")))]
            GeoConfig::RipeGeo { path, overlaps } => {
                RipeGeoImpl::from_folder(&path, overlaps.unwrap_or(RipeGeoOverlapsStrategy::Skip))?
                    .into()
            }
            #[cfg(feature = "ripe-geo-embedded")]
            GeoConfig::RipeGeo {
                path: Some(path),
                overlaps,
            } => {
                RipeGeoImpl::from_folder(&path, overlaps.unwrap_or(RipeGeoOverlapsStrategy::Skip))?
                    .into()
            }
            #[cfg(feature = "ripe-geo-embedded")]
            GeoConfig::RipeGeo {
                path: None,
                overlaps: _,
            } => RipeGeoImpl::from_embedded().into(),
        };
        Ok(slf)
    }
}

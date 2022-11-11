pub use continent::Continent;
pub use error::GeoError;
#[cfg(feature = "ripe-geo")]
use ripe_geo::{config::RipeGeoConfig, RipeGeo, RipeGeoImpl};

mod continent;
mod error;
#[cfg(feature = "maxminddb")]
pub mod max_mind_db;
#[cfg(feature = "ripe-geo")]
pub mod ripe_geo;

use enum_dispatch::enum_dispatch;
use serde::Deserialize;
use std::net::IpAddr;
#[cfg(feature = "maxminddb")]
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

#[cfg(feature = "ripe-geo")]
impl From<RipeGeoImpl> for Geo {
    fn from(value: RipeGeoImpl) -> Self {
        Geo::RipeGeo(value.into())
    }
}

#[enum_dispatch(Geo)]
pub trait GeoTrait: Send + Sync {
    fn try_lookup_continent(&self, address: IpAddr) -> Result<Continent, GeoError>;
    fn start_autoupdate(&self) -> bool;
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
enum GeoConfig {
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
    RipeGeo(RipeGeoConfig),
}

impl TryFrom<GeoConfig> for Geo {
    type Error = GeoError;

    fn try_from(value: GeoConfig) -> Result<Self, Self::Error> {
        match value {
            #[cfg(feature = "maxminddb")]
            GeoConfig::MaxMindDb { path } => Ok(Self::MaxMindDb(
                max_mind_db::MaxMindDbGeo::from_file(&path)?,
            )),
            #[cfg(feature = "ripe-geo")]
            GeoConfig::RipeGeo(config) => {
                let ripe_geo: RipeGeo = config.try_into()?;
                Ok(ripe_geo.into())
            }
        }
    }
}

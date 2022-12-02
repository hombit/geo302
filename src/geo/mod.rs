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

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
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
    RipeGeo(RipeGeoConfig),
}

impl GeoConfig {
    pub fn load(self) -> Result<Geo, GeoError> {
        match self {
            #[cfg(feature = "maxminddb")]
            Self::MaxMindDb { path } => {
                let geo = Geo::MaxMindDb(max_mind_db::MaxMindDbGeo::from_file(&path)?);
                log::info!("Maxmind DB is loaded from {path:?}");
                Ok(geo)
            }
            #[cfg(feature = "ripe-geo")]
            Self::RipeGeo(config) => {
                let ripe_geo: RipeGeo = config.try_into()?;
                Ok(ripe_geo.into())
            }
        }
    }
}

#[cfg(feature = "ripe-geo")]
use crate::geo::ripe_geo::RipeGeoDataError;

#[cfg(feature = "maxminddb")]
use maxminddb::MaxMindDBError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoError {
    #[error("continent is not recognised")]
    ContinentUnknown,
    #[cfg(feature = "maxminddb")]
    #[error(transparent)]
    MaxMindDBError(#[from] MaxMindDBError),
    #[cfg(feature = "ripe-geo")]
    #[error(transparent)]
    RipeGeo(#[from] RipeGeoDataError),
    #[error("No ripe-geo data available: embedded data is not compiled in, no path to data specified, autoupdate is disabled or not supported in this build")]
    RipeGeoConfigNoPath,
}

use crate::geo::ripe_geo::RipeGeoDataError;

use maxminddb::MaxMindDBError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoError {
    #[error("continent is not recognised")]
    ContinentUnknown,
    #[error(transparent)]
    MaxMindDBError(#[from] MaxMindDBError),
    #[error(transparent)]
    RipeGeo(#[from] RipeGeoDataError),
}

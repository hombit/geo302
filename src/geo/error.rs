use maxminddb::MaxMindDBError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoError {
    #[error("continent is not recognised")]
    ContinentUnknown,
    #[error(transparent)]
    MaxMindDBError(#[from] MaxMindDBError),
}

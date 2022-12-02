#![allow(dead_code)]

use serde::Deserialize;
use thiserror::Error;

/// This is a helper struct to mark config items as non-available for the current Cargo build
/// feature set.
#[derive(Debug, Deserialize, Default)]
#[serde(try_from = "UnavailableDe")]
pub struct Unavailable;

#[derive(Debug, Deserialize)]
struct UnavailableDe(toml::Value);

#[derive(Debug, Error)]
#[error("This configuration item is not available for this build, please rebuild with appropriate Cargo build features")]
struct UnavailableError;

impl TryFrom<UnavailableDe> for Unavailable {
    type Error = UnavailableError;

    fn try_from(_: UnavailableDe) -> Result<Self, Self::Error> {
        Err(UnavailableError)
    }
}

use crate::geo::{Continent, GeoError};

use std::net::IpAddr;

pub trait Geo: Send + Sync {
    fn try_lookup_continent(&self, address: IpAddr) -> Result<Continent, GeoError>;
}

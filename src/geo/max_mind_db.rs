use crate::geo::{Continent, Geo, GeoError};

use maxminddb::geoip2;
use std::net::IpAddr;
use std::path::Path;

struct GeoNameId(pub u32);

impl From<u32> for GeoNameId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl TryInto<Continent> for GeoNameId {
    type Error = GeoError;

    fn try_into(self) -> Result<Continent, GeoError> {
        match self.0 {
            6255146_u32 => Ok(Continent::Africa),
            6255147_u32 => Ok(Continent::Asia),
            6255148_u32 => Ok(Continent::Europe),
            6255149_u32 => Ok(Continent::NorthAmerica),
            6255151_u32 => Ok(Continent::Oceania),
            6255150_u32 => Ok(Continent::SouthAmerica),
            6255152_u32 => Ok(Continent::Antarctica),
            _ => Err(GeoError::ContinentUnknown),
        }
    }
}

pub struct MaxMindDbGeo {
    maxminddb_reader: maxminddb::Reader<Vec<u8>>,
}

impl MaxMindDbGeo {
    pub fn from_file<P: AsRef<Path>>(filepath: P) -> Result<Self, GeoError> {
        Ok(Self {
            maxminddb_reader: maxminddb::Reader::open_readfile(filepath)?,
        })
    }
}

impl Geo for MaxMindDbGeo {
    fn try_lookup_continent(&self, address: IpAddr) -> Result<Continent, GeoError> {
        let country: geoip2::Country = self.maxminddb_reader.lookup(address)?;
        let geo_name_id: GeoNameId = country
            .continent
            .ok_or(GeoError::ContinentUnknown)?
            .geoname_id
            .ok_or(GeoError::ContinentUnknown)?
            .into();
        geo_name_id.try_into()
    }
}

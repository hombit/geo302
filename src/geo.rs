use maxminddb::{geoip2, MaxMindDBError};
use std::net::IpAddr;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Continent {
    Africa,
    Asia,
    Europe,
    NorthAmerica,
    Oceania,
    SouthAmerica,
    Antarctica,
    Default,
}

impl TryFrom<u32> for Continent {
    type Error = GeoError;

    fn try_from(geoname_id: u32) -> Result<Self, GeoError> {
        match geoname_id {
            6255146_u32 => Ok(Self::Africa),
            6255147_u32 => Ok(Self::Asia),
            6255148_u32 => Ok(Self::Europe),
            6255149_u32 => Ok(Self::NorthAmerica),
            6255151_u32 => Ok(Self::Oceania),
            6255150_u32 => Ok(Self::SouthAmerica),
            6255152_u32 => Ok(Self::Antarctica),
            _ => Err(GeoError::ContinentUnknown),
        }
    }
}

impl<'a> TryFrom<&'a str> for Continent {
    type Error = GeoError;

    fn try_from(s: &'a str) -> Result<Self, GeoError> {
        match s.trim() {
            "Africa" => Ok(Self::Africa),
            "Asia" => Ok(Self::Asia),
            "Europe" => Ok(Self::Europe),
            "NorthAmerica" => Ok(Self::NorthAmerica),
            "Oceania" => Ok(Self::Oceania),
            "SouthAmerica" => Ok(Self::SouthAmerica),
            "Antarctica" => Ok(Self::Antarctica),
            "default" => Ok(Self::Default),
            _ => Err(GeoError::ContinentUnknown),
        }
    }
}

impl From<Continent> for &'static str {
    fn from(continent: Continent) -> Self {
        match continent {
            Continent::Africa => "Africa",
            Continent::Asia => "Asia",
            Continent::Europe => "Europe",
            Continent::NorthAmerica => "North America",
            Continent::Oceania => "Oceania",
            Continent::SouthAmerica => "South America",
            Continent::Antarctica => "Antarctica",
            Continent::Default => "default",
        }
    }
}

#[derive(Error, Debug)]
pub enum GeoError {
    #[error("continent is not recognised")]
    ContinentUnknown,
    #[error(transparent)]
    MaxMindDBError(#[from] MaxMindDBError),
}

pub struct Geo {
    maxminddb_reader: maxminddb::Reader<Vec<u8>>,
}

impl Geo {
    pub fn from_file<P: AsRef<Path>>(filepath: P) -> Result<Self, GeoError> {
        Ok(Self {
            maxminddb_reader: maxminddb::Reader::open_readfile(filepath)?,
        })
    }

    pub fn try_lookup_continent(&self, address: IpAddr) -> Result<Continent, GeoError> {
        let country: geoip2::Country = self.maxminddb_reader.lookup(address)?;
        country
            .continent
            .ok_or(GeoError::ContinentUnknown)?
            .geoname_id
            .ok_or(GeoError::ContinentUnknown)?
            .try_into()
    }
}

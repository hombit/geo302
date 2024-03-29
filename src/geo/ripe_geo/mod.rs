use crate::geo::{Continent, GeoError, GeoTrait};
use crate::intervals::{IntervalBTreeMap, IntervalVec, Intervals};

use serde::Deserialize;
use std::collections::HashSet;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Read};
use std::net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr};
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::str::{FromStr, Utf8Error};
#[cfg(feature = "ripe-geo-autoupdate")]
use std::sync::{Arc, RwLock};
use thiserror::Error;

pub mod config;
#[cfg(feature = "ripe-geo-embedded")]
pub mod embedded;
#[cfg(feature = "ripe-geo-autoupdate")]
pub mod updater;

#[derive(Copy, Clone, Deserialize, Debug)]
pub enum RipeGeoOverlapsStrategy {
    #[serde(alias = "fail")]
    Fail,
    #[serde(alias = "skip")]
    Skip,
}

impl Default for RipeGeoOverlapsStrategy {
    fn default() -> Self {
        Self::Skip
    }
}

#[derive(Error, Debug)]
pub enum RipeGeoDataError {
    #[error(r#"Error parsing file "{path}": {error}"#)]
    FileCorrupted {
        path: PathBuf,
        error: RipeGeoFileError,
    },
    #[error(r#"Some files are missed: {0:?}"#)]
    MissingFiles(HashSet<(Continent, IpType)>),
    #[error(r#"Error while attemping to read directory "{path}": {error}"#)]
    DirIoError {
        path: PathBuf,
        error: std::io::Error,
    },
    #[error(r#"Eror while attemping to read file "{path}": {error}"#)]
    FileIoError {
        path: PathBuf,
        error: std::io::Error,
    },
    #[cfg(feature = "ripe-geo-autoupdate")]
    #[error(r#"Error while reading file "{path}" of tar.gz archive: {error}"#)]
    ArchiveEntryIoError {
        path: PathBuf,
        error: std::io::Error,
    },
    #[cfg(feature = "ripe-geo-autoupdate")]
    #[error(r"Error while reading tar.gz file: {0}")]
    ArchiveReadError(std::io::Error),
    #[cfg(feature = "ripe-geo-autoupdate")]
    #[error(transparent)]
    DownloadError(#[from] updater::RipeGeoDownloadError),
}

#[derive(Error, Debug)]
pub enum RipeGeoFileError {
    #[error("Not a valid UTF8 file")]
    Utf8Error(#[from] Utf8Error),
    #[error(transparent)]
    FileReadError(#[from] std::io::Error),
    #[error(r#"Record "{record}" is invalid: {error:?}"#)]
    InvalidRecord {
        record: String,
        error: RipeGeoRecordError,
    },
    #[error(r#"Record "{0}" overlaps with previously inserted "{1}""#)]
    OverlappedRecord(String, String),
    #[error("File is empty")]
    EmptyFile,
}

#[derive(Error, Debug)]
pub enum RipeGeoRecordError {
    #[error("Record must be in format SUBNET/SUFFIX")]
    Parts,
    #[error("Subnet of the record has wrong format")]
    Subnet(#[from] AddrParseError),
    #[error("Suffix of the record has wrong format")]
    SuffixFormat(#[from] ParseIntError),
    #[error(r#"Suffix of the record is too large: "{0}""#)]
    SuffixTooLarge(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IpType {
    V4,
    V6,
}

trait IpTypeTrait {
    type Addr: FromStr<Err = AddrParseError> + From<Self::UInt> + Copy + std::fmt::Debug;
    type UInt: From<Self::Addr>
        + Copy
        + Ord
        + std::fmt::Debug
        + std::ops::Add<Self::UInt, Output = Self::UInt>;
    const BITS: u32;

    fn size_from_suffix(suffix: u32) -> Option<Self::UInt>;

    fn suffix_from_size(size: Self::UInt) -> Option<u32>;
}

struct IpV4;

impl IpTypeTrait for IpV4 {
    type Addr = Ipv4Addr;
    type UInt = u32;
    const BITS: u32 = u32::BITS;

    fn size_from_suffix(suffix: u32) -> Option<Self::UInt> {
        Some((1 as Self::UInt) << (Self::BITS.checked_sub(suffix)? as Self::UInt))
    }

    fn suffix_from_size(size: Self::UInt) -> Option<u32> {
        if size.count_ones() == 1 {
            Some(size.leading_zeros() + 1)
        } else {
            None
        }
    }
}

struct IpV6;

impl IpTypeTrait for IpV6 {
    type Addr = Ipv6Addr;
    type UInt = u128;
    const BITS: u32 = u128::BITS;

    fn size_from_suffix(suffix: u32) -> Option<Self::UInt> {
        Some((1 as Self::UInt) << (Self::BITS.checked_sub(suffix)? as Self::UInt))
    }

    fn suffix_from_size(size: Self::UInt) -> Option<u32> {
        if size.count_ones() == 1 {
            Some(size.leading_zeros() + 1)
        } else {
            None
        }
    }
}

impl<'a> TryFrom<&'a str> for IpType {
    type Error = ();

    fn try_from(s: &'a str) -> Result<Self, ()> {
        match s {
            "ipv4" => Ok(Self::V4),
            "ipv6" => Ok(Self::V6),
            _ => Err(()),
        }
    }
}

struct Record<Ip>
where
    Ip: IpTypeTrait,
{
    subnet: Ip::Addr,
    size: Ip::UInt,
}

impl<Ip> FromStr for Record<Ip>
where
    Ip: IpTypeTrait,
{
    type Err = RipeGeoRecordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (subnet, suffix) = s.split_once('/').ok_or(RipeGeoRecordError::Parts)?;
        let subnet: Ip::Addr = subnet.parse()?;
        let suffix: u32 = suffix.parse()?;
        let size =
            Ip::size_from_suffix(suffix).ok_or(RipeGeoRecordError::SuffixTooLarge(suffix))?;
        Ok(Self { subnet, size })
    }
}

impl<Ip> ToString for Record<Ip>
where
    Ip: IpTypeTrait,
{
    fn to_string(&self) -> String {
        format!(
            "{subnet:?}/{suffix:?}",
            subnet = self.subnet,
            suffix = Ip::suffix_from_size(self.size).expect("size must be power of two")
        )
    }
}

const ALL_RIPE_GEO_CONTINENTS: [Continent; 6] = [
    Continent::Africa,
    Continent::Asia,
    Continent::Europe,
    Continent::NorthAmerica,
    Continent::Oceania,
    Continent::SouthAmerica,
];

#[cfg(feature = "ripe-geo-autoupdate")]
pub struct RipeGeo {
    inner: Arc<RwLock<RipeGeoImpl>>,
    overlaps_strategy: RipeGeoOverlapsStrategy,
    updater: Option<RwLock<updater::RipeGeoUpdater>>,
}

#[cfg(not(feature = "ripe-geo-autoupdate"))]
pub struct RipeGeo(RipeGeoImpl);

impl From<RipeGeoImpl> for RipeGeo {
    fn from(value: RipeGeoImpl) -> Self {
        #[cfg(feature = "ripe-geo-autoupdate")]
        {
            Self {
                inner: Arc::new(RwLock::new(value)),
                overlaps_strategy: RipeGeoOverlapsStrategy::default(),
                updater: None,
            }
        }
        #[cfg(not(feature = "ripe-geo-autoupdate"))]
        {
            Self(value)
        }
    }
}

impl GeoTrait for RipeGeo {
    fn try_lookup_continent(&self, address: IpAddr) -> Result<Continent, GeoError> {
        #[cfg(feature = "ripe-geo-autoupdate")]
        {
            self.inner.read().unwrap().try_lookup_continent(address)
        }
        #[cfg(not(feature = "ripe-geo-autoupdate"))]
        {
            self.0.try_lookup_continent(address)
        }
    }

    fn start_autoupdate(&self) -> bool {
        #[cfg(feature = "ripe-geo-autoupdate")]
        {
            let mut updater = match &self.updater {
                Some(value) => value,
                None => return false,
            }
            .write()
            .unwrap();
            updater.start(self).is_some()
        }
        #[cfg(not(feature = "ripe-geo-autoupdate"))]
        {
            false
        }
    }
}

pub struct RipeGeoImpl {
    ipv4: IntervalVec<u32, Continent>,
    ipv6: IntervalVec<u128, Continent>,
}

impl RipeGeoImpl {
    /// Parse paths like "asia.ipv4.list"
    fn parse_path(path: &Path) -> Option<(Continent, IpType)> {
        let continent_ip_str = match path.file_name()?.to_str()?.rsplit_once('.') {
            Some((s, "list")) => s,
            _ => return None,
        };
        let (continent, ip) = continent_ip_str.split_once('.')?;
        let continent: Continent = continent.try_into().ok()?;
        let ip: IpType = ip.try_into().ok()?;
        Some((continent, ip))
    }

    fn insert_file<Ip>(
        tree: &mut IntervalBTreeMap<Ip::UInt, Continent>,
        reader: Box<dyn Read>,
        continent: Continent,
        overlaps_strategy: RipeGeoOverlapsStrategy,
    ) -> Result<Vec<RipeGeoFileError>, RipeGeoFileError>
    where
        Ip: IpTypeTrait,
    {
        let buf_reader = BufReader::new(reader);
        let mut warnings = vec![];
        let mut count = 0;
        for line in buf_reader.lines() {
            let line = line?;
            let record: Record<Ip> =
                line.parse()
                    .map_err(|error| RipeGeoFileError::InvalidRecord {
                        record: line,
                        error,
                    })?;
            let subnet_numeric: Ip::UInt = record.subnet.into();
            if let Err(error) = tree.try_insert(subnet_numeric, record.size, continent) {
                let error = RipeGeoFileError::OverlappedRecord(
                    record.to_string(),
                    Record::<Ip> {
                        subnet: error.key.into(),
                        size: error.size,
                    }
                    .to_string(),
                );
                match overlaps_strategy {
                    RipeGeoOverlapsStrategy::Fail => return Err(error),
                    RipeGeoOverlapsStrategy::Skip => warnings.push(error),
                }
            } else {
                count += 1;
            }
        }
        if count == 0 {
            Err(RipeGeoFileError::EmptyFile)
        } else {
            Ok(warnings)
        }
    }

    pub fn from_text_files<I, P>(
        it: I,
        overlaps_strategy: RipeGeoOverlapsStrategy,
    ) -> Result<Self, RipeGeoDataError>
    where
        I: Iterator<Item = Result<(P, Box<dyn Read>), RipeGeoDataError>>,
        P: AsRef<Path>,
    {
        let mut ipv4 = IntervalBTreeMap::new();
        let mut ipv6 = IntervalBTreeMap::new();
        let mut cont_ip_set = {
            let mut set = HashSet::new();
            for continent in ALL_RIPE_GEO_CONTINENTS {
                for ip in [IpType::V4, IpType::V6] {
                    set.insert((continent, ip));
                }
            }
            set
        };
        for result in it {
            let (path, reader) = result?;
            let path = path.as_ref();
            let (continent, ip) = match Self::parse_path(path) {
                Some(value) => value,
                None => continue,
            };
            cont_ip_set.remove(&(continent, ip));
            let error_mapper = |error| RipeGeoDataError::FileCorrupted {
                error,
                path: path.to_owned(),
            };
            match ip {
                IpType::V4 => {
                    Self::insert_file::<IpV4>(&mut ipv4, reader, continent, overlaps_strategy)
                }
                IpType::V6 => {
                    Self::insert_file::<IpV6>(&mut ipv6, reader, continent, overlaps_strategy)
                }
            }
            .map_err(error_mapper)?
            .into_iter()
            .map(error_mapper)
            .for_each(|warning| log::warn!("{warning}"));
        }
        if !cont_ip_set.is_empty() {
            return Err(RipeGeoDataError::MissingFiles(cont_ip_set));
        }
        Ok(Self {
            ipv4: ipv4.into(),
            ipv6: ipv6.into(),
        })
    }

    pub fn from_folder(
        dir_path: &Path,
        overlaps_strategy: RipeGeoOverlapsStrategy,
    ) -> Result<Self, RipeGeoDataError> {
        let it = std::fs::read_dir(dir_path)
            .map_err(|error| RipeGeoDataError::DirIoError {
                error,
                path: dir_path.to_owned(),
            })?
            .filter_map(|entry| {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        return Some(Err(RipeGeoDataError::DirIoError {
                            error,
                            path: dir_path.to_owned(),
                        }))
                    }
                };
                let path = entry.path();
                let file_type = match entry.file_type() {
                    Ok(file_type) => file_type,
                    Err(error) => return Some(Err(RipeGeoDataError::FileIoError { error, path })),
                };
                if !file_type.is_file() {
                    return None;
                }
                let file = match std::fs::File::open(&path) {
                    Ok(file) => file,
                    Err(error) => return Some(Err(RipeGeoDataError::FileIoError { error, path })),
                };
                let boxed_file: Box<dyn Read> = Box::new(file);
                Some(Ok((path, boxed_file)))
            });
        Self::from_text_files(it, overlaps_strategy)
    }

    pub fn into_interval_btree_maps(
        self,
    ) -> (
        IntervalBTreeMap<u32, Continent>,
        IntervalBTreeMap<u128, Continent>,
    ) {
        (self.ipv4.into(), self.ipv6.into())
    }

    pub fn into_interval_vecs(self) -> (IntervalVec<u32, Continent>, IntervalVec<u128, Continent>) {
        (self.ipv4, self.ipv6)
    }

    pub fn into_intervals(self) -> (Intervals<u32, Continent>, Intervals<u128, Continent>) {
        (self.ipv4.into(), self.ipv6.into())
    }
}

impl RipeGeoImpl {
    fn try_lookup_continent(&self, address: IpAddr) -> Result<Continent, GeoError> {
        match address {
            IpAddr::V4(ip) => self.ipv4.get(ip.into()),
            IpAddr::V6(ip) => self.ipv6.get(ip.into()),
        }
        .ok_or(GeoError::ContinentUnknown)
        .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_to_from_string_ipv4() {
        let s = "37.228.128.0/23";
        let record: Record<IpV4> = s.parse().unwrap();
        assert_eq!(record.to_string(), s);
    }

    #[test]
    fn record_to_from_string_ipv6() {
        let s = "2001:43f8:700::/44";
        let record: Record<IpV6> = s.parse().unwrap();
        assert_eq!(record.to_string(), s);
    }
}

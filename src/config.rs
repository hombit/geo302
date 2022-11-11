use crate::geo::Geo;
#[allow(unused_imports)]
use crate::unavailable::Unavailable;
use crate::Mirror;

use hyper::HeaderMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::num::NonZeroU64;
#[cfg(feature = "multi-thread")]
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::path::Path;
use std::time::Duration;

#[derive(Deserialize)]
#[serde(from = "NonZeroU64")]
pub struct HealthCheckInterval(Duration);

impl Default for HealthCheckInterval {
    fn default() -> Self {
        Self(Duration::from_secs(5))
    }
}

impl Deref for HealthCheckInterval {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<NonZeroU64> for HealthCheckInterval {
    fn from(value: NonZeroU64) -> Self {
        Self(Duration::from_secs(value.get()))
    }
}

impl From<HealthCheckInterval> for Duration {
    fn from(value: HealthCheckInterval) -> Self {
        value.0
    }
}

#[cfg(feature = "multi-thread")]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum ConfigThreads {
    Custom(NonZeroUsize),
    #[serde(alias = "cores", alias = "cpus")]
    Cores,
}

#[cfg(feature = "multi-thread")]
impl Default for ConfigThreads {
    fn default() -> Self {
        Self::Custom(2.try_into().unwrap())
    }
}

#[cfg(not(feature = "multi-thread"))]
type ConfigThreads = Unavailable;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "Config::default_host")]
    pub host: SocketAddr,
    #[serde(default = "Config::default_ip_headers")]
    pub ip_headers: Vec<String>,
    #[serde(default = "Config::default_ip_headers_recursive")]
    pub ip_headers_recursive: bool,
    #[serde(default)]
    pub healthckeck_interval: HealthCheckInterval,
    #[serde(default, with = "http_serde::header_map")]
    pub response_headers: HeaderMap,
    #[serde(default = "Config::default_log_level")]
    pub log_level: log::Level,
    #[serde(default)]
    pub threads: ConfigThreads,
    pub geoip: Geo,
    pub mirrors: HashMap<String, Mirror>,
    pub continents: HashMap<String, Vec<String>>,
}

impl Config {
    fn default_host() -> SocketAddr {
        "127.0.0.1:8080".parse().unwrap()
    }

    fn default_ip_headers() -> Vec<String> {
        vec!["X-FORWARDED-FOR".into()]
    }

    fn default_ip_headers_recursive() -> bool {
        true
    }

    fn default_log_level() -> log::Level {
        log::Level::Info
    }
}

pub fn parse_config<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
    let toml_string = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&toml_string)?;
    Ok(config)
}

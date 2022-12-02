use crate::geo::GeoConfig;
use crate::healthcheck::HealthCheckConfig;
use crate::mirror::Mirror;
#[cfg(not(feature = "multi-thread"))]
use crate::unavailable::Unavailable;

use hyper::HeaderMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
#[cfg(feature = "multi-thread")]
use std::num::NonZeroUsize;
use std::path::Path;

#[cfg(feature = "multi-thread")]
#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "Config::default_host")]
    pub host: SocketAddr,
    #[serde(default = "Config::default_ip_headers")]
    pub ip_headers: Vec<String>,
    #[serde(default = "Config::default_ip_headers_recursive")]
    pub ip_headers_recursive: bool,
    #[serde(default)]
    pub healthcheck: HealthCheckConfig,
    #[serde(default, with = "http_serde::header_map")]
    pub response_headers: HeaderMap,
    #[serde(default = "Config::default_log_level")]
    pub log_level: log::Level,
    #[serde(default)]
    pub threads: ConfigThreads,
    pub geoip: GeoConfig,
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

#[cfg(test)]
mod tests {
    use super::*;

    use include_dir::{include_dir, Dir};

    const CONFIG_EXAMPLES: Dir = include_dir!("$CARGO_MANIFEST_DIR/config-examples");

    #[cfg(all(
        feature = "maxminddb",
        feature = "ripe-geo-embedded",
        feature = "ripe-geo-autoupdate"
    ))]
    #[test]
    fn load_all_config_examples() {
        for entry in CONFIG_EXAMPLES.find("**/*.toml").unwrap() {
            let file = entry.as_file().unwrap();
            let toml_string = file.contents_utf8().unwrap();
            let result: Result<Config, _> = toml::from_str(toml_string);
            assert!(result.is_ok(), "{result:?}");
        }
    }

    fn load_from_example_config(s: &'static str) -> Result<Config, toml::de::Error> {
        let file = CONFIG_EXAMPLES.get_file(s).unwrap();
        let toml_string = file.contents_utf8().unwrap();
        toml::from_str(toml_string)
    }

    macro_rules! load_config {
        ($name: ident, $file: expr, $($feature: expr $(,)?)+) => {
            #[test]
            fn $name() {
                #[allow(unused_variables)]
                let result = load_from_example_config($file);
                #[cfg(all(
                    $(
                        feature = $feature,
                    )*
                ))]
                assert!(result.is_ok(), "must be Ok, got: {result:?}");
                #[cfg(not(any(
                    $(
                        feature = $feature,
                    )*
                )))]
                assert!(result.is_err(), "must be Err, got: {result:?}");
            }
        };
    }

    load_config!(load_maxminddb_config, "maxmind-db.toml", "maxminddb");

    load_config!(
        load_ripe_geo_autoupdate_no_dir_1,
        "ripe-geo-autoupdate-no-dir-1.toml",
        "ripe-geo-autoupdate"
    );
    load_config!(
        load_ripe_geo_autoupdate_no_dir_2,
        "ripe-geo-autoupdate-no-dir-2.toml",
        "ripe-geo-autoupdate"
    );
    load_config!(
        load_ripe_geo_autoupdate_no_dir_3,
        "ripe-geo-autoupdate-no-dir-3.toml",
        "ripe-geo-autoupdate"
    );
    load_config!(
        load_ripe_geo_autoupdate_no_dir_4,
        "ripe-geo-autoupdate-no-dir-4.toml",
        "ripe-geo-autoupdate"
    );

    // Negative test doesn't work well here
    #[cfg(feature = "ripe-geo-embedded")]
    #[test]
    fn load_ripe_geo_embedded_no_autoupdate_1() {
        let result: Result<Config, _> =
            load_from_example_config("ripe-geo-embedded-no-autoupdate-1.toml");
        assert!(result.is_ok());
    }
    load_config!(
        load_ripe_geo_embedded_no_autoupdate_2,
        "ripe-geo-embedded-no-autoupdate-2.toml",
        "ripe-geo-autoupdate",
        "ripe-geo-embedded"
    );
    load_config!(
        ripe_geo_embedded_no_autoupdate_missed_healthcheck,
        "ripe-geo-embedded-no-autoupdate-missed-healthcheck.toml",
        "ripe-geo-autoupdate",
        "ripe-geo-embedded"
    );

    load_config!(
        load_ripe_geo_from_dir_and_autoupdate_1,
        "ripe-geo-from-dir-and-autoupdate-1.toml",
        "ripe-geo-autoupdate"
    );
    load_config!(
        load_ripe_geo_from_dir_and_autoupdate_2,
        "ripe-geo-from-dir-and-autoupdate-2.toml",
        "ripe-geo-autoupdate"
    );
    load_config!(
        load_ripe_geo_from_dir_and_autoupdate_3,
        "ripe-geo-from-dir-and-autoupdate-3.toml",
        "ripe-geo-autoupdate"
    );
    load_config!(
        load_ripe_geo_from_dir_and_autoupdate_4,
        "ripe-geo-from-dir-and-autoupdate-4.toml",
        "ripe-geo-autoupdate"
    );
    load_config!(
        load_ripe_geo_from_dir_and_autoupdate_5,
        "ripe-geo-from-dir-and-autoupdate-5.toml",
        "ripe-geo-autoupdate"
    );

    load_config!(
        load_ripe_geo_from_dir_no_autoupdate_1,
        "ripe-geo-from-dir-no-autoupdate-1.toml",
        "ripe-geo"
    );
    load_config!(
        load_ripe_geo_from_dir_no_autoupdate_2,
        "ripe-geo-from-dir-no-autoupdate-2.toml",
        "ripe-geo-autoupdate"
    );
}

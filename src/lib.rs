#[cfg(not(any(feature = "maxminddb", feature = "ripe-geo")))]
compile_error!("At least one of geo-IP database features must be enabled");

// Remove after IpAddr::to_canonical stabilizes
// https://github.com/rust-lang/rust/issues/27709
mod canonical_ip;
pub mod config;
mod geo;
mod header_tools;
mod healthcheck;
#[cfg(feature = "ripe-geo")]
mod interval_tree;
mod mirror;
pub mod service;
mod unavailable;
mod uri_tools;

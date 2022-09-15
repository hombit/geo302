use crate::config::parse_config;
use crate::geo::{Continent, Geo};
use crate::healthcheck::check_health;
use crate::mirror::{ContinentMap, Mirror, MirrorVec};
use crate::rejects::{handle_rejection, MirrorsUnavailable};
use crate::uri_tools::compose_uri;

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use warp::http::header::{
    HeaderMap, HeaderName, HeaderValue, InvalidHeaderName, InvalidHeaderValue,
};
use warp::path::FullPath;
use warp::reply::Reply;
use warp::Filter;

mod config;
mod filters;
mod geo;
mod healthcheck;
mod mirror;
mod rejects;
mod uri_tools;

#[derive(Error, Debug)]
pub enum HeaderError {
    #[error("{0}")]
    InvalidHeaderName(InvalidHeaderName),
    #[error("{0}")]
    InvalidHeaderValue(InvalidHeaderValue),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "geo302.toml".to_owned());

    let config = parse_config(config_path)?;
    let host: SocketAddr = config.host.parse()?;
    let ip_header_recursive = config.ip_headers_recursive;
    let continent_map = ContinentMap::from_config(&config)?;
    let geo = Arc::new(Geo::from_config(&config)?);
    let check_interval = Duration::new(config.healthckeck_interval.get().into(), 0);
    let ip_header_names = config.ip_headers;
    let response_headers = config
        .response_headers
        .iter()
        .map(|(name, value)| {
            let name = match HeaderName::from_str(name) {
                Ok(name) => name,
                Err(err) => return Err(HeaderError::InvalidHeaderName(err)),
            };
            let value = match HeaderValue::from_str(value) {
                Ok(value) => value,
                Err(err) => return Err(HeaderError::InvalidHeaderValue(err)),
            };
            Ok((name, value))
        })
        .collect::<Result<HeaderMap, _>>()?;

    simple_logger::init_with_level(config.log_level)?;

    check_health(continent_map.all_mirrors(), check_interval);

    let logs = warp::log::custom(|info| {
        log::info!(
            "{} {} {} {}",
            info.remote_addr()
                .map_or_else(|| "_".into(), |addr| format!("{}", addr.ip())),
            info.method(),
            info.path(),
            info.status(),
        )
    });

    let routes = warp::get()
        .and(filters::client_ip(ip_header_names, ip_header_recursive))
        .map(
            move |ip: Option<IpAddr>| match ip.and_then(|ip| geo.try_lookup_continent(ip).ok()) {
                Some(continent) => continent_map.get(continent),
                None => continent_map.get_default(),
            },
        )
        .and(warp::path::full())
        .and(
            warp::query::raw()
                .map(Option::Some)
                .or_else(|_err| async { Ok::<_, warp::Rejection>((None,)) }),
        )
        .and_then(
            |mirrors: MirrorVec, path: FullPath, query: Option<String>| async move {
                let mirror = {
                    let mut it_mirrors = mirrors.iter();
                    loop {
                        match it_mirrors.next() {
                            Some(mirror) => {
                                if mirror.available.load(Ordering::Acquire) {
                                    break mirror;
                                }
                            }
                            None => return Err(warp::reject::custom(MirrorsUnavailable)),
                        }
                    }
                };
                let uri = compose_uri(
                    &mirror.upstream,
                    &(String::from(path.as_str())
                        + query.as_ref().map(String::as_str).unwrap_or("")),
                )
                .expect("Invalid URI"); // TODO: handle error

                Ok(warp::redirect::found(uri).into_response())
            },
        )
        .with(warp::filters::reply::headers(response_headers))
        .recover(handle_rejection)
        .with(logs);

    warp::serve(routes).run(host).await;
    Ok(())
}

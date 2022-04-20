use crate::config::parse_config;
use crate::geo::{Continent, Geo};
use crate::healthcheck::check_health;
use crate::mirror::{ContinentMap, Mirror, MirrorVec};
use crate::rejects::{handle_rejection, BrokenPath, MirrorsUnavailable};
use filters::client_ip_filter;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use warp::http::Uri;
use warp::path::FullPath;
use warp::Filter;

mod config;
mod filters;
mod geo;
mod healthcheck;
mod mirror;
mod rejects;

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
        .and(client_ip_filter(ip_header_names, ip_header_recursive))
        .map(
            move |ip: Option<IpAddr>| match ip.and_then(|ip| geo.try_lookup_continent(ip).ok()) {
                Some(continent) => continent_map.get(continent),
                None => continent_map.get_default(),
            },
        )
        .and(warp::path::full())
        .and_then(|mirrors: MirrorVec, path: FullPath| async move {
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
            let url = mirror
                .upstream
                .join(path.as_str().trim_start_matches('/'))
                .map_err(|_| warp::reject::custom(BrokenPath))?;
            Ok(warp::redirect::found(url.as_str().parse::<Uri>().unwrap()))
        })
        .recover(handle_rejection)
        .with(logs);

    warp::serve(routes).run(host).await;
    Ok(())
}

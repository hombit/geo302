use crate::config::parse_config;
use crate::geo::{Continent, Geo};
use crate::mirror::{ContinentMap, Mirror, MirrorVec};
use reqwest::Client;
use reqwest::StatusCode;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use warp::http::{HeaderMap, Uri};
use warp::path::FullPath;
use warp::Filter;

mod config;
mod geo;
mod mirror;

#[derive(Debug)]
struct MirrorsUnavailable;

impl warp::reject::Reject for MirrorsUnavailable {}

async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    if err.find::<MirrorsUnavailable>().is_some() {
        Ok(warp::reply::with_status(
            "SERVICE_UNAVAILABLE",
            StatusCode::SERVICE_UNAVAILABLE,
        ))
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        Ok(warp::reply::with_status(
            "INTERNAL_SERVER_ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
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
    let ip_header_names = config.ip_headers;

    let http_client = Client::new();
    let http_client_filter = warp::any().map(move || http_client.clone());

    let client_ip_filter = {
        warp::header::headers_cloned()
            .and(warp::filters::addr::remote())
            .map(move |headers: HeaderMap, socket_addr: Option<SocketAddr>| {
                ip_header_names
                    .iter()
                    .filter_map(|name| {
                        let values = headers.get_all(name);
                        let mut it_values = values.iter();
                        if ip_header_recursive {
                            it_values.next()
                        } else {
                            it_values.next_back()
                        }
                    })
                    .next()
                    .and_then(|value| {
                        let value = value.to_str().ok()?;
                        let mut split = value.split(',');
                        if ip_header_recursive {
                            split.next()
                        } else {
                            split.next_back()
                        }
                    })
                    .and_then(|s| s.parse::<IpAddr>().ok())
                    .or_else(|| socket_addr.as_ref().map(SocketAddr::ip))
            })
    };

    let routes = warp::get()
        .and(client_ip_filter)
        .map(
            move |ip: Option<IpAddr>| match ip.and_then(|ip| geo.try_lookup_continent(ip).ok()) {
                Some(continent) => continent_map.get(continent),
                None => continent_map.get_default(),
            },
        )
        .and(warp::path::full())
        .and(http_client_filter)
        .and_then(
            |mirrors: MirrorVec, path: FullPath, client: Client| async move {
                let mirror = {
                    let mut it_mirrors = mirrors.iter();
                    loop {
                        match it_mirrors.next() {
                            Some(mirror) => {
                                if let Ok(response) =
                                    client.get(mirror.healthcheck.clone()).send().await
                                {
                                    if response.status() == StatusCode::OK {
                                        break mirror;
                                    }
                                }
                            }
                            None => return Err(warp::reject::custom(MirrorsUnavailable)),
                        }
                    }
                };
                let url = mirror
                    .upstream
                    .join(path.as_str().trim_start_matches('/'))
                    .unwrap();
                Ok(warp::redirect::found(url.as_str().parse::<Uri>().unwrap()))
            },
        )
        .recover(handle_rejection);

    warp::serve(routes).run(host).await;
    Ok(())
}

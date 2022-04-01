use crate::config::parse_config;
use crate::geo::{Continent, Geo};
use crate::mirror::{ContinentMap, Mirror, MirrorVec};
use reqwest::Client;
use reqwest::StatusCode;
use std::net::SocketAddr;
use std::sync::Arc;
use warp::http::Uri;
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
    if let Some(_) = err.find::<MirrorsUnavailable>() {
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
    let continent_map = ContinentMap::from_config(&config)?;
    let geo = Arc::new(Geo::from_config(&config)?);

    let client = Client::new();
    let client_filter = warp::any().map(move || client.clone());
    let routes = warp::get()
        .and(warp::filters::addr::remote())
        .map(move |addr: Option<SocketAddr>| {
            let ip = match addr {
                Some(addr) => addr.ip(),
                None => return continent_map.get_default(),
            };
            match geo.try_lookup_continent(ip) {
                Ok(continent) => continent_map.get(continent),
                Err(_) => continent_map.get_default(),
            }
        })
        .and(warp::path::full())
        .and(client_filter)
        .and_then(
            |mirrors: MirrorVec, path: FullPath, client: Client| async move {
                let mirror = {
                    let mut it_mirrors = mirrors.into_iter();
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

use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use warp::http::HeaderMap;
use warp::Filter;

pub fn client_ip_filter(
    header_names: Vec<String>,
    recursive: bool,
) -> impl Filter<Extract = (Option<IpAddr>,), Error = Infallible> + Clone + 'static {
    warp::header::headers_cloned()
        .and(warp::filters::addr::remote())
        .map(move |headers: HeaderMap, socket_addr: Option<SocketAddr>| {
            header_names
                .iter()
                .filter_map(|name| {
                    let values = headers.get_all(name);
                    let mut it_values = values.iter();
                    if recursive {
                        it_values.next()
                    } else {
                        it_values.next_back()
                    }
                })
                .next()
                .and_then(|value| {
                    let value = value.to_str().ok()?;
                    let mut split = value.split(',');
                    if recursive {
                        split.next()
                    } else {
                        split.next_back()
                    }
                })
                .and_then(|s| s.parse::<IpAddr>().ok())
                .or_else(|| socket_addr.as_ref().map(SocketAddr::ip))
        })
}

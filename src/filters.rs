use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use warp::http::HeaderMap;
use warp::Filter;

pub fn client_ip(
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
                .and_then(|s| s.trim().parse::<IpAddr>().ok())
                .or_else(|| socket_addr.as_ref().map(SocketAddr::ip))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[tokio::test]
    async fn client_ip_filter_no_addr() {
        let actual = warp::test::request()
            .filter(&client_ip(vec![], true))
            .await
            .unwrap();
        assert_eq!(actual, None);
    }

    #[tokio::test]
    async fn client_ip_filter_socket_addr_v4() {
        let ip_expected = IpAddr::V4(Ipv4Addr::new(93, 180, 26, 112));
        let actual = warp::test::request()
            .remote_addr(SocketAddr::new(ip_expected, 8001))
            .filter(&client_ip(vec![], true))
            .await
            .unwrap();
        assert_eq!(actual, Some(ip_expected));
    }

    #[tokio::test]
    async fn client_ip_filter_socket_addr_v6() {
        let ip_expected = IpAddr::V6(Ipv6Addr::new(
            0x2600, 0x1f18, 0x1f, 0xdb01, 0x11af, 0x58af, 0xae11, 0xf645,
        ));
        let actual = warp::test::request()
            .remote_addr(SocketAddr::new(ip_expected, 8001))
            .filter(&client_ip(vec![], true))
            .await
            .unwrap();
        assert_eq!(actual, Some(ip_expected));
    }

    #[tokio::test]
    async fn client_ip_filter_single_header() {
        let ip_expected = IpAddr::V4(Ipv4Addr::new(128, 174, 199, 60));
        let ip_socket = IpAddr::V4(Ipv4Addr::new(93, 180, 26, 112));
        let header = "X-FORWARDED-FOR";

        for is_recursive in [false, true] {
            let actual = warp::test::request()
                .remote_addr(SocketAddr::new(ip_socket, 8001))
                .header(header, ip_expected.to_string())
                .filter(&client_ip(vec![header.into()], is_recursive))
                .await
                .unwrap();
            assert_eq!(actual, Some(ip_expected));
        }
    }

    #[tokio::test]
    async fn client_ip_filter_one_header_two_values() {
        let ip_client = IpAddr::V4(Ipv4Addr::new(128, 174, 199, 60));
        let ip_proxy = IpAddr::V4(Ipv4Addr::new(80, 94, 184, 70));
        let ip_socket = IpAddr::V4(Ipv4Addr::new(93, 180, 26, 112));
        let header_name = "X-FORWARDED-FOR";
        let header_value = format!("{}, {}", ip_client, ip_proxy);

        for (ip_expected, is_recursive) in [(ip_proxy, false), (ip_client, true)] {
            let actual = warp::test::request()
                .remote_addr(SocketAddr::new(ip_socket, 8001))
                .header(header_name, &header_value)
                .filter(&client_ip(vec![header_name.into()], is_recursive))
                .await
                .unwrap();
            assert_eq!(actual, Some(ip_expected));
        }
    }

    #[tokio::test]
    async fn client_ip_filter_one_header_three_values() {
        let ip_client = IpAddr::V4(Ipv4Addr::new(128, 174, 199, 60));
        let ip_proxy1 = IpAddr::V4(Ipv4Addr::new(80, 94, 184, 70));
        let ip_proxy2 = IpAddr::V4(Ipv4Addr::new(52, 0, 14, 116));
        let ip_socket = IpAddr::V4(Ipv4Addr::new(93, 180, 26, 112));
        let header_name = "X-FORWARDED-FOR";
        let header_value = format!("{}, {}, {}", ip_client, ip_proxy1, ip_proxy2);

        for (ip_expected, is_recursive) in [(ip_proxy2, false), (ip_client, true)] {
            let actual = warp::test::request()
                .remote_addr(SocketAddr::new(ip_socket, 8001))
                .header(header_name, &header_value)
                .filter(&client_ip(vec![header_name.into()], is_recursive))
                .await
                .unwrap();
            assert_eq!(actual, Some(ip_expected));
        }
    }

    #[tokio::test]
    async fn client_ip_filter_two_headers() {
        let ip_client = IpAddr::V4(Ipv4Addr::new(128, 174, 199, 60));
        let ip_nginx_proxy = IpAddr::V4(Ipv4Addr::new(80, 94, 184, 70));
        let ip_other_proxy = IpAddr::V4(Ipv4Addr::new(80, 94, 184, 70));
        let ip_socket = IpAddr::V4(Ipv4Addr::new(93, 180, 26, 112));
        let headers = [
            ("X-REAL-IP", format!("{}, {}", ip_client, ip_nginx_proxy)),
            (
                "X-FORWARDED-FOR",
                format!("{}, {}", ip_nginx_proxy, ip_other_proxy),
            ),
        ];
        let header_names: Vec<_> = headers
            .iter()
            .map(|(name, _value)| name.to_string())
            .collect();

        for (ip_expected, is_recursive) in [(ip_nginx_proxy, false), (ip_client, true)] {
            let mut request = warp::test::request().remote_addr(SocketAddr::new(ip_socket, 8001));
            for (name, value) in &headers {
                request = request.header(*name, value);
            }
            let actual = request
                .filter(&client_ip(header_names.clone(), is_recursive))
                .await
                .unwrap();
            assert_eq!(actual, Some(ip_expected));
        }
    }
}

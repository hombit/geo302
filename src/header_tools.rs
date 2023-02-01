use hyper::HeaderMap;
use std::net::IpAddr;

pub fn client_ip(headers: &HeaderMap, header_names: &[String], recursive: bool) -> Option<IpAddr> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn client_ip_filter_no_addr() {
        for is_recursive in [false, true] {
            let actual = client_ip(&HeaderMap::new(), &[], is_recursive);
            assert_eq!(actual, None);
        }
    }

    #[test]
    fn client_ip_filter_single_header() {
        let ip_expected = IpAddr::V4(Ipv4Addr::new(128, 174, 199, 60));
        let header_name = "X-FORWARDED-FOR";
        let request_headers = {
            let mut headers = HeaderMap::new();
            headers.insert(header_name, ip_expected.to_string().parse().unwrap());
            headers
        };

        for is_recursive in [false, true] {
            let actual = client_ip(&request_headers, &[header_name.to_string()], is_recursive);
            assert_eq!(actual, Some(ip_expected));
        }
    }

    #[test]
    fn client_ip_filter_one_header_two_values() {
        let ip_client = IpAddr::V4(Ipv4Addr::new(128, 174, 199, 60));
        let ip_proxy = IpAddr::V4(Ipv4Addr::new(80, 94, 184, 70));
        let ip_header_name = "X-FORWARDED-FOR";
        let request_header_value = format!("{ip_client}, {ip_proxy}");
        let request_headers = {
            let mut headers = HeaderMap::new();
            headers.insert(ip_header_name, request_header_value.parse().unwrap());
            headers
        };

        for (ip_expected, is_recursive) in [(ip_proxy, false), (ip_client, true)] {
            let actual = client_ip(
                &request_headers,
                &[ip_header_name.to_string()],
                is_recursive,
            );
            assert_eq!(actual, Some(ip_expected));
        }
    }

    #[tokio::test]
    async fn client_ip_filter_one_header_three_values() {
        let ip_client = IpAddr::V4(Ipv4Addr::new(128, 174, 199, 60));
        let ip_proxy1 = IpAddr::V4(Ipv4Addr::new(80, 94, 184, 70));
        let ip_proxy2 = IpAddr::V4(Ipv4Addr::new(52, 0, 14, 116));
        let ip_header_name = "X-FORWARDED-FOR";
        let request_header_value = format!("{ip_client}, {ip_proxy1}, {ip_proxy2}");
        let request_headers = {
            let mut headers = HeaderMap::new();
            headers.insert(ip_header_name, request_header_value.parse().unwrap());
            headers
        };

        for (ip_expected, is_recursive) in [(ip_proxy2, false), (ip_client, true)] {
            let actual = client_ip(
                &request_headers,
                &[ip_header_name.to_string()],
                is_recursive,
            );
            assert_eq!(actual, Some(ip_expected));
        }
    }

    #[test]
    fn client_ip_filter_two_headers() {
        let ip_client = IpAddr::V4(Ipv4Addr::new(128, 174, 199, 60));
        let ip_nginx_proxy = IpAddr::V4(Ipv4Addr::new(80, 94, 184, 70));
        let ip_other_proxy = IpAddr::V4(Ipv4Addr::new(80, 94, 184, 70));
        let request_headers = {
            let mut headers = HeaderMap::new();
            headers.insert(
                "X-REAL-IP",
                format!("{ip_client}, {ip_nginx_proxy}").parse().unwrap(),
            );
            headers.insert(
                "X-FORWARDED-FOR",
                format!("{ip_nginx_proxy}, {ip_other_proxy}")
                    .parse()
                    .unwrap(),
            );
            headers
        };
        let header_names: Vec<_> = request_headers
            .keys()
            .map(|name| name.as_str().to_string())
            .collect();

        for (ip_expected, is_recursive) in [(ip_nginx_proxy, false), (ip_client, true)] {
            let actual = client_ip(&request_headers, &header_names, is_recursive);
            assert_eq!(actual, Some(ip_expected));
        }
    }
}

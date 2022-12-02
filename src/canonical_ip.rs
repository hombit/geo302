/// Implementation of unstable IpAddr::to_canonical
/// https://github.com/rust-lang/rust/issues/27709
use std::net::IpAddr;

pub trait CanonicalIpAddr {
    fn to_canonical_ip(&self) -> Self;
}

impl CanonicalIpAddr for IpAddr {
    fn to_canonical_ip(&self) -> Self {
        match self {
            IpAddr::V4(v4) => IpAddr::V4(*v4),
            IpAddr::V6(v6) => match v6.to_ipv4() {
                Some(v4) => IpAddr::V4(v4),
                None => IpAddr::V6(*v6),
            },
        }
    }
}

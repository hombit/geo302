use crate::canonical_ip::CanonicalIpAddr;
use crate::config::Config;
use crate::geo::{Geo, GeoError, GeoTrait};
use crate::header_tools::client_ip;
use crate::healthcheck::HealthCheck;
use crate::mirror::{ContinentMap, ContinentMapConfigError, Mirror};
use crate::uri_tools::compose_uri;

use hyper::{header::HeaderMap, Body, Request, Response, StatusCode, Uri};
use std::net::IpAddr;
use std::sync::atomic::Ordering;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("No available mirrors")]
    MirrorsUnavailable,
    #[error("Requested URI {0:?} is invalid")]
    InvalidUri(Uri),
    #[error(r#"Internal server error: "{0:?}""#)]
    InternalServerError(#[from] hyper::http::Error),
}

pub struct Geo302Service {
    ip_headers: Vec<String>,
    ip_headers_recursive: bool,
    response_headers: HeaderMap,
    geo: Geo,
    continent_map: ContinentMap,
    #[allow(dead_code)] // We need HealthCheck only for its side effects
    health_check: HealthCheck,
}

impl Geo302Service {
    pub fn from_config(config: Config) -> Result<Self, InvalidConfigError> {
        let Config {
            ip_headers,
            ip_headers_recursive,
            response_headers,
            healthcheck: health_check_config,
            geoip: geo_config,
            mirrors: conf_mirrors,
            continents: conf_continents,
            ..
        } = config;

        let continent_map =
            ContinentMap::from_mirrors_and_continents(&conf_mirrors, &conf_continents)?;

        let health_check = health_check_config.start(continent_map.all_mirrors());

        let geo = geo_config.load()?;
        geo.start_autoupdate();

        Ok(Self {
            ip_headers,
            ip_headers_recursive,
            response_headers,
            geo,
            continent_map,
            health_check,
        })
    }
}

impl Geo302Service {
    fn mirror(&self, remote_ip: IpAddr) -> Result<Mirror, ServiceError> {
        let mirrors = match self.geo.try_lookup_continent(remote_ip).ok() {
            Some(continent) => self.continent_map.get(continent),
            None => self.continent_map.get_default(),
        };
        let mut it_mirrors = mirrors.iter();
        loop {
            match it_mirrors.next() {
                Some(mirror) => {
                    if mirror.available.load(Ordering::Acquire) {
                        return Ok(mirror.clone());
                    }
                }
                None => {
                    return Err(ServiceError::MirrorsUnavailable);
                }
            }
        }
    }

    fn remote_ip(&self, headers: &HeaderMap, socket_ip_addr: IpAddr) -> IpAddr {
        client_ip(headers, &self.ip_headers, self.ip_headers_recursive).unwrap_or(socket_ip_addr)
    }

    pub fn response(
        &self,
        socket_ip_addr: IpAddr,
        request: &Request<Body>,
    ) -> Result<Response<Body>, ServiceError> {
        let remote_ip = self
            .remote_ip(request.headers(), socket_ip_addr)
            .to_canonical_ip();
        let mirror = self.mirror(remote_ip)?;
        let request_path = request
            .uri()
            .path_and_query()
            .ok_or_else(|| ServiceError::InvalidUri(request.uri().clone()))?;
        let uri = compose_uri(&mirror.upstream, request_path.as_str())?;
        let response = {
            let mut response_builder = Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", uri.to_string());
            {
                let headers = response_builder.headers_mut().unwrap();
                for (name, value) in &self.response_headers {
                    headers.insert(name, value.clone());
                }
            }
            response_builder.body(Body::empty())?
        };
        Ok(response)
    }
}

pub fn make_error_response(error: ServiceError) -> Response<Body> {
    let status = match error {
        ServiceError::MirrorsUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        ServiceError::InvalidUri(_) => StatusCode::BAD_REQUEST,
        ServiceError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    Response::builder()
        .status(status)
        .body(format!("{error:?}").into())
        .unwrap()
}

pub fn log_response(socket_ip_addr: IpAddr, request: &Request<Body>, response: &Response<Body>) {
    log::info!(
        "{} {} {} {} {}",
        socket_ip_addr,
        request.method(),
        request.uri(),
        response.status(),
        response
            .headers()
            .get("Location")
            .map(|header_value| header_value.to_str().unwrap_or("-"))
            .unwrap_or("-"),
    );
}

#[derive(Debug, Error)]
pub enum InvalidConfigError {
    #[error(transparent)]
    ContinentMapConfigError(#[from] ContinentMapConfigError),
    #[error(transparent)]
    GeoError(#[from] GeoError),
}

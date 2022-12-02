use crate::mirror::Mirror;
use crate::non_zero_duration::NonZeroDuration;

use hyper::client::Client;
use hyper::http::uri::Uri;
use hyper::StatusCode;
use hyper_tls::HttpsConnector;
use serde::Deserialize;

use std::sync::atomic;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
enum RequestError {
    #[error(transparent)]
    Http(#[from] hyper::Error),
    #[error("Connection timeout")]
    Timeout(#[from] tokio::time::error::Elapsed),
}

#[derive(Debug, Deserialize, Clone)]
pub struct HealthCheckConfig {
    #[serde(default = "HealthCheckConfig::default_interval")]
    interval: NonZeroDuration,
    #[serde(default = "HealthCheckConfig::default_timeout")]
    timeout: NonZeroDuration,
}

pub struct HealthCheck {
    #[allow(dead_code)] // we don't use it now
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl HealthCheckConfig {
    async fn get_status<C>(
        client: &Client<C>,
        uri: Uri,
        timeout: Duration,
    ) -> Result<StatusCode, RequestError>
    where
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        let response = tokio::time::timeout(timeout, client.get(uri)).await??;
        Ok(response.status())
    }

    pub fn start(self, mirrors: &[Mirror]) -> HealthCheck {
        let https = HttpsConnector::new();
        let http_client = Client::builder().build::<_, hyper::Body>(https);
        let handles = mirrors
            .iter()
            .map(|mirror| {
                let http_client = http_client.clone();
                let mirror = mirror.clone();
                let HealthCheckConfig { interval, timeout } = self.clone();
                let interval = interval.into();
                let timeout = timeout.into();
                tokio::spawn(async move {
                    loop {
                        let status =
                            Self::get_status(&http_client, mirror.healthcheck.clone(), timeout)
                                .await;
                        // Use Result.is_ok_and when stabilizes
                        // https://github.com/rust-lang/rust/issues/93050
                        let new_available = match status {
                            Ok(success) => success.is_success(),
                            Err(_) => false,
                        };
                        mirror
                            .available
                            .store(new_available, atomic::Ordering::Release);
                        match (new_available, status) {
                            (true, Ok(_)) => log::info!("{} is alive", mirror.healthcheck),
                            (false, Ok(status)) => {
                                log::warn!("{} is unavailable: {}", mirror.healthcheck, status)
                            }
                            (_, Err(e)) => {
                                log::warn!(
                                    "{} is unavailable: {}",
                                    mirror.healthcheck,
                                    e.to_string()
                                )
                            }
                        }
                        tokio::time::sleep(interval).await;
                    }
                })
            })
            .collect();
        HealthCheck { handles }
    }
}

impl HealthCheckConfig {
    fn default_interval() -> NonZeroDuration {
        NonZeroDuration::from_secs(5).unwrap()
    }

    fn default_timeout() -> NonZeroDuration {
        NonZeroDuration::from_secs(3).unwrap()
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Self::default_interval(),
            timeout: Self::default_timeout(),
        }
    }
}

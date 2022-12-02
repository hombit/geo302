use crate::mirror::Mirror;

use hyper::client::Client;
use hyper::http::uri::Uri;
use hyper::StatusCode;
use hyper_tls::HttpsConnector;
use std::sync::atomic;
use std::time::Duration;
use thiserror::Error;

const TIMEOUT: Duration = Duration::new(5, 0);

#[derive(Debug, Error)]
enum RequestError {
    #[error(transparent)]
    Http(#[from] hyper::Error),
    #[error("Connection timeout")]
    Timeout(#[from] tokio::time::error::Elapsed),
}

pub struct HealthCheck {
    #[allow(dead_code)] // we don't use it now
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl HealthCheck {
    async fn get_status<C>(client: &Client<C>, uri: Uri) -> Result<StatusCode, RequestError>
    where
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        let response = tokio::time::timeout(TIMEOUT, client.get(uri)).await??;
        Ok(response.status())
    }

    pub fn start(mirrors: &[Mirror], interval: Duration) -> Self {
        let https = HttpsConnector::new();
        let http_client = Client::builder().build::<_, hyper::Body>(https);
        let handles = mirrors
            .iter()
            .map(|mirror| {
                let http_client = http_client.clone();
                let mirror = mirror.clone();
                tokio::spawn(async move {
                    loop {
                        let status =
                            Self::get_status(&http_client, mirror.healthcheck.clone()).await;
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
        Self { handles }
    }
}

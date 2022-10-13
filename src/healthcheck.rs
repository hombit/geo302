use crate::Mirror;

use hyper::client::Client;
use hyper_tls::HttpsConnector;
use std::sync::{atomic, Arc};
use std::time::Duration;

pub struct HealthCheck {}

impl HealthCheck {
    pub fn start(mirrors: &[Mirror], interval: Duration) -> Self {
        let https = HttpsConnector::new();
        let http_client = Client::builder().build::<_, hyper::Body>(https);
        for mirror in mirrors {
            let available = Arc::clone(&mirror.available);
            let healthcheck_url = mirror.healthcheck.clone();
            let http_client = http_client.clone();
            tokio::spawn(async move {
                loop {
                    let status = http_client
                        .get(healthcheck_url.clone())
                        .await
                        .map(|response| response.status());
                    // Use Result.is_ok_and when stabilizes
                    // https://github.com/rust-lang/rust/issues/93050
                    let new_available = match status {
                        Ok(success) => success.is_success(),
                        Err(_) => false,
                    };
                    available.store(new_available, atomic::Ordering::Release);
                    match (new_available, status) {
                        (true, Ok(_)) => log::info!("{} is alive", healthcheck_url),
                        (false, Ok(status)) => {
                            log::warn!("{} is unavailable: {}", healthcheck_url, status)
                        }
                        (_, Err(e)) => {
                            log::warn!("{} is unavailable: {}", healthcheck_url, e.to_string())
                        }
                    }
                    tokio::time::sleep(interval).await;
                }
            });
        }
        Self {}
    }
}

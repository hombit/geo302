use crate::Mirror;

use hyper::client::Client;
use hyper_tls::HttpsConnector;
use std::sync::atomic;
use std::time::Duration;

pub struct HealthCheck {}

impl HealthCheck {
    pub fn start(mirrors: &[Mirror], interval: Duration) -> Self {
        let https = HttpsConnector::new();
        let http_client = Client::builder().build::<_, hyper::Body>(https);
        for mirror in mirrors {
            let http_client = http_client.clone();
            let mirror = mirror.clone();
            tokio::spawn(async move {
                loop {
                    let status = http_client
                        .get(mirror.healthcheck.clone())
                        .await
                        .map(|response| response.status());
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
                            log::warn!("{} is unavailable: {}", mirror.healthcheck, e.to_string())
                        }
                    }
                    tokio::time::sleep(interval).await;
                }
            });
        }
        Self {}
    }
}

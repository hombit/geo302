use crate::Mirror;
use reqwest::Client;
use std::sync::{atomic, Arc};
use std::time::Duration;

pub fn check_health(mirrors: &[Mirror], interval: Duration) {
    let http_client = Client::new();
    for mirror in mirrors {
        let available = Arc::clone(&mirror.available);
        let healthcheck_url = mirror.healthcheck.clone();
        let http_client = http_client.clone();
        tokio::spawn(async move {
            loop {
                let status = http_client
                    .get(healthcheck_url.clone())
                    .send()
                    .await
                    .map(|response| response.error_for_status());
                available.store(status.is_ok(), atomic::Ordering::Release);
                match status {
                    Ok(_) => log::info!("{} is alive", healthcheck_url),
                    Err(e) => {
                        log::warn!("{} is unavailable: {}", healthcheck_url, e.to_string())
                    }
                }
                tokio::time::sleep(interval).await;
            }
        });
    }
}

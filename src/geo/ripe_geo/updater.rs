use super::*;

use core::num::NonZeroU64;
use hyper::body::{Body, Bytes};
use hyper::client::connect::Connect;
use hyper::client::Client;
use hyper::http::uri::Uri;
use hyper::StatusCode;
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use std::io::Cursor;
use std::time::Duration;

#[derive(Debug, Error)]
pub enum RipeGeoDownloadError {
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Http(#[from] hyper::http::Error),
    #[error("Non-success status code: {0}")]
    NonSuccess(StatusCode),
    #[error("Error while unpacking downloaded tar.gz: {0}")]
    UnpackIo(#[from] std::io::Error),
}

impl From<StatusCode> for RipeGeoDownloadError {
    fn from(status_code: StatusCode) -> Self {
        RipeGeoDownloadError::NonSuccess(status_code)
    }
}

lazy_static! {
    static ref RIPE_GEO_URL: Uri =
        "https://github.com/hombit/ripe-geo-history/archive/refs/heads/continents.tar.gz"
            .parse()
            .unwrap();
}

const RIPE_GEO_UPDATE_INTERVAL_SECONDS: u64 = 86400;

#[derive(Deserialize)]
#[serde(from = "RipeGeoUpdaterConfig")]
pub struct RipeGeoUpdater {
    interval: Duration,
    uri: Uri,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl RipeGeoUpdater {
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn default_uri() -> Uri {
        RIPE_GEO_URL.clone()
    }

    pub fn default_uri_ref() -> &'static Uri {
        &RIPE_GEO_URL
    }

    pub fn default_interval() -> Duration {
        Duration::from_secs(RIPE_GEO_UPDATE_INTERVAL_SECONDS)
    }
}

impl Default for RipeGeoUpdater {
    fn default() -> Self {
        Self::new(Self::default_interval(), Self::default_uri())
    }
}

#[derive(Deserialize)]
struct RipeGeoUpdaterConfig {
    #[serde(default)]
    interval: UpdaterIntervalConfig,
    #[serde(
        default = "RipeGeoUpdater::default_uri",
        alias = "url",
        with = "http_serde::uri"
    )]
    uri: Uri,
}

impl From<RipeGeoUpdaterConfig> for RipeGeoUpdater {
    fn from(config: RipeGeoUpdaterConfig) -> Self {
        Self {
            interval: config.interval.0,
            uri: config.uri,
            handle: None,
        }
    }
}

#[derive(Deserialize)]
#[serde(from = "NonZeroU64")]
struct UpdaterIntervalConfig(Duration);

impl Default for UpdaterIntervalConfig {
    fn default() -> Self {
        Self(RipeGeoUpdater::default_interval())
    }
}

impl From<NonZeroU64> for UpdaterIntervalConfig {
    fn from(value: NonZeroU64) -> Self {
        Self(Duration::from_secs(value.get()))
    }
}

impl RipeGeoUpdater {
    pub fn new(interval: Duration, uri: Uri) -> Self {
        Self {
            interval,
            uri,
            handle: None,
        }
    }

    pub fn start(&mut self, ripe_geo: &RipeGeo) -> Option<&tokio::task::JoinHandle<()>> {
        if self.handle.is_some() {
            return None;
        }

        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, Body>(https);
        let overlaps_strategy = ripe_geo.overlaps_strategy;
        let ripe_geo_impl_lock = ripe_geo.inner.clone();
        let uri = self.uri.clone();
        let interval = self.interval;

        self.handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let new_ripe_geo_impl =
                    match RipeGeoImpl::download(&client, &uri, overlaps_strategy).await {
                        Ok(val) => val,
                        Err(err) => {
                            log::warn!(
                                r#"Error while attempting to update ripe-geo from "{uri}": {err}"#,
                            );
                            continue;
                        }
                    };
                {
                    let mut ripe_geo_impl = ripe_geo_impl_lock.write().unwrap();
                    let _ = std::mem::replace(
                        std::ops::DerefMut::deref_mut(&mut ripe_geo_impl),
                        new_ripe_geo_impl,
                    );
                }
                log::info!(r#"ripe-geo database updated from "{uri}""#);
            }
        })
        .into();
        self.handle.as_ref()
    }
}

impl RipeGeo {
    pub fn set_updater(&mut self, updater: Option<RipeGeoUpdater>) {
        self.updater = updater.map(RwLock::new);
    }
}

impl RipeGeoImpl {
    async fn download_archive<C>(
        client: &Client<C>,
        mut uri: Uri,
    ) -> Result<Bytes, RipeGeoDownloadError>
    where
        C: Connect + Clone + Send + Sync + 'static,
    {
        const MAX_ATTEMPTS: usize = 8;
        let mut attempt = 0;
        let response = loop {
            let request = hyper::Request::builder().uri(&uri).body(Body::empty())?;
            let response = client.request(request).await?;

            if response.status().is_success() {
                break response;
            } else if response.status().is_redirection() {
                uri = response
                    .headers()
                    .get("Location")
                    .ok_or_else(|| response.status())?
                    .as_bytes()
                    .try_into()
                    .map_err(|_| response.status())?;
            } else {
                return Err(response.status().into());
            }

            attempt += 1;
            if attempt == MAX_ATTEMPTS {
                return Err(response.status().into());
            }
        };
        let body = response.into_body();
        Ok(hyper::body::to_bytes(body).await?)
    }

    pub async fn download<C>(
        client: &Client<C>,
        uri: &Uri,
        overlaps_strategy: RipeGeoOverlapsStrategy,
    ) -> Result<Self, RipeGeoDataError>
    where
        C: Connect + Clone + Send + Sync + 'static,
    {
        let body = Self::download_archive(client, uri.clone()).await?;
        let gz_reader = flate2::bufread::GzDecoder::new(body.as_ref());
        let mut tar_archive = tar::Archive::new(gz_reader);
        let it = tar_archive
            .entries()
            .map_err(RipeGeoDataError::ArchiveReadError)?
            .filter_map(|entry| {
                let mut entry = entry.ok()?;
                let path = entry.path().ok()?;
                let path = path.into_owned();
                let mut vec = Vec::with_capacity(entry.size() as usize);
                if let Err(error) = entry.read_to_end(&mut vec) {
                    return Some(Err(RipeGeoDataError::ArchiveEntryIoError { path, error }));
                }
                let boxed_entry: Box<dyn Read> = Box::new(Cursor::new(vec));
                Some(Ok((path, boxed_entry)))
            });
        Self::from_text_files(it, overlaps_strategy)
    }

    pub fn from_uri(
        uri: &Uri,
        overlaps_strategy: RipeGeoOverlapsStrategy,
    ) -> Result<Self, RipeGeoDataError> {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, Body>(https);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(Self::download(&client, uri, overlaps_strategy))
    }
}

#[cfg(feature = "ripe-geo-autoupdate")]
use super::updater::RipeGeoUpdater;
use super::*;
#[cfg(not(feature = "ripe-geo-autoupdate"))]
use crate::unavailable::Unavailable;

#[cfg(feature = "ripe-geo-autoupdate")]
use hyper::http::Uri;

#[derive(Deserialize)]
pub struct RipeGeoConfig {
    #[serde(default)]
    path: Option<PathBuf>,
    #[serde(default)]
    overlaps: RipeGeoOverlapsStrategy,
    #[serde(default)]
    autoupdate: RipeGeoAutoupdateConfig,
}

impl RipeGeoConfig {
    /// Parse config with respect to Cargo features
    /// - Load from path is specified
    /// - If autoupdate is enabled, download from web
    /// - If not (or download failed), but embedded is enabled, load from binary
    /// - Return error otherwise
    fn ripe_geo_impl(&self) -> Result<RipeGeoImpl, GeoError> {
        // autoupdate could be unused
        #[allow(unused_variables)]
        let Self {
            path,
            overlaps,
            autoupdate,
        } = self;
        // We would like to move to the None branch when this stabilizes
        // https://github.com/rust-lang/rust/issues/15701
        #[allow(unreachable_code)]
        match path {
            Some(path) => Ok(RipeGeoImpl::from_folder(path, *overlaps)?),
            None => {
                #[cfg(feature = "ripe-geo-autoupdate")]
                let from_url = {
                    let uri = autoupdate.uri().ok_or(GeoError::RipeGeoConfigNoPath)?;
                    RipeGeoImpl::from_uri(uri, *overlaps)
                };
                #[cfg(feature = "ripe-geo-embedded")]
                {
                    #[cfg(feature = "ripe-geo-autoupdate")]
                    if let Ok(ripe_geo_impl) = from_url {
                        return Ok(ripe_geo_impl);
                    }
                    return Ok(RipeGeoImpl::from_embedded());
                }
                #[cfg(feature = "ripe-geo-autoupdate")]
                return from_url.map_err(Into::into);
                Err(GeoError::RipeGeoConfigNoPath)
            }
        }
    }
}

impl TryInto<RipeGeo> for RipeGeoConfig {
    type Error = GeoError;

    fn try_into(self) -> Result<RipeGeo, Self::Error> {
        #[allow(unused_mut)]
        let mut ripe_geo: RipeGeo = self.ripe_geo_impl()?.into();
        #[cfg(feature = "ripe-geo-autoupdate")]
        {
            ripe_geo.set_updater(self.autoupdate.into_updater())
        }
        Ok(ripe_geo)
    }
}

#[cfg(feature = "ripe-geo-autoupdate")]
#[derive(Deserialize)]
#[serde(untagged)]
enum RipeGeoAutoupdateConfig {
    Boolean(bool),
    Updater(RipeGeoUpdater),
}

#[cfg(feature = "ripe-geo-autoupdate")]
impl Default for RipeGeoAutoupdateConfig {
    fn default() -> Self {
        Self::Boolean(false)
    }
}

#[cfg(feature = "ripe-geo-autoupdate")]
impl RipeGeoAutoupdateConfig {
    fn uri(&self) -> Option<&Uri> {
        match self {
            Self::Boolean(false) => None,
            Self::Boolean(true) => Some(RipeGeoUpdater::default_uri_ref()),
            Self::Updater(updater) => Some(updater.uri()),
        }
    }

    fn into_updater(self) -> Option<RipeGeoUpdater> {
        match self {
            Self::Boolean(false) => None,
            Self::Boolean(true) => Some(RipeGeoUpdater::default()),
            Self::Updater(updater) => Some(updater),
        }
    }
}

#[cfg(not(feature = "ripe-geo-autoupdate"))]
type RipeGeoAutoupdateConfig = Unavailable;

use super::*;

#[cfg(feature = "ripe-geo-embedded")]
use include_dir::include_dir;

#[cfg(feature = "ripe-geo-embedded")]
const RIPE_GEO_CONTINENTS_DIR: include_dir::Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/ripe-geo/continents");

impl RipeGeoImpl {
    // Since all errors here are related to compile-time issues, we don't need Result and just panic
    pub fn from_embedded() -> Self {
        let it = RIPE_GEO_CONTINENTS_DIR.files().map(|file| {
            let reader: Box<dyn Read> = Box::new(file.contents());
            Ok((file.path(), reader))
        });
        Self::from_text_files(it, RipeGeoOverlapsStrategy::Skip)
            .expect("Recompile geo302 with correct ripe-geo/continents folder embedded")
    }
}

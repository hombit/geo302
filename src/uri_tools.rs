use hyper::http::{Error, Uri};

pub fn compose_uri(base_uri: &Uri, path: &str) -> Result<Uri, Error> {
    let new_path = [base_uri.path(), path].concat();
    Uri::builder()
        .scheme(base_uri.scheme().unwrap().clone())
        .authority(base_uri.authority().unwrap().clone())
        .path_and_query(new_path)
        .build()
}

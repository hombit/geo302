use warp::http::StatusCode;

#[derive(Debug)]
pub struct MirrorsUnavailable;

impl warp::reject::Reject for MirrorsUnavailable {}

pub async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    if err.find::<MirrorsUnavailable>().is_some() {
        Ok(warp::reply::with_status(
            "SERVICE_UNAVAILABLE",
            StatusCode::SERVICE_UNAVAILABLE,
        ))
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        Ok(warp::reply::with_status(
            "INTERNAL_SERVER_ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

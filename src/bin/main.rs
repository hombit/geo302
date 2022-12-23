#[cfg(feature = "multi-thread")]
use geo302::config::ConfigThreads;
use geo302::config::{parse_config, Config};
use geo302::service::{log_response, make_error_response, Geo302Service, InvalidConfigError};

use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Server};
use std::convert::Infallible;
use std::sync::Arc;

async fn async_main(config: Config) -> anyhow::Result<()> {
    let host = config.host;

    simple_logger::init_with_level(config.log_level)?;

    let geo302_service = tokio::task::spawn_blocking(move || -> Result<_, InvalidConfigError> {
        Ok(Arc::new(Geo302Service::from_config(config)?))
    })
    .await??;

    let make_service = make_service_fn(move |connection: &AddrStream| {
        let socket_remote_ip = connection.remote_addr().ip();
        let geo302_service = geo302_service.clone();
        let service = service_fn(move |request: Request<Body>| {
            let geo302_service = geo302_service.clone();
            async move {
                let response = geo302_service
                    .response(socket_remote_ip, &request)
                    .unwrap_or_else(make_error_response);
                log_response(socket_remote_ip, &request, &response);
                Ok::<_, Infallible>(response)
            }
        });
        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&host).serve(make_service);

    if let Err(e) = server.await {
        log::error!("server error: {}", e);
    }
    Err(anyhow::anyhow!("server exited"))
}

fn main() -> anyhow::Result<()> {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "geo302.toml".to_owned());

    let config = parse_config(&config_path)?;

    #[cfg(feature = "multi-thread")]
    let mut runtime_builder = match config.threads {
        ConfigThreads::Custom(threads) => match threads.into() {
            1 => tokio::runtime::Builder::new_current_thread(),
            threads => {
                let mut builder = tokio::runtime::Builder::new_multi_thread();
                builder.worker_threads(threads);
                builder
            }
        },
        ConfigThreads::Cores => tokio::runtime::Builder::new_multi_thread(),
    };
    #[cfg(not(feature = "multi-thread"))]
    let mut runtime_builder = tokio::runtime::Builder::new_current_thread();
    let runtime = runtime_builder.enable_all().build().unwrap();

    runtime.block_on(async_main(config))
}

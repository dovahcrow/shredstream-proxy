use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use log::{error, info};
use once_cell::sync::Lazy;
use prometheus::{gather, register_int_counter, Encoder, IntCounter, TextEncoder};
use tokio::time::sleep;

pub async fn serve_metrics(addr: SocketAddr, shutdown: Arc<AtomicBool>) {
    let make_service = make_service_fn(|_| async {
        Ok::<_, Infallible>(service_fn(|_req| async {
            let mut buffer = vec![];
            let encoder = TextEncoder::new();
            let metric_families = gather();
            encoder.encode(&metric_families, &mut buffer).unwrap();

            Ok::<_, Infallible>(Response::new(Body::from(buffer)))
        }))
    });

    let server = Server::bind(&addr)
        .serve(make_service)
        .with_graceful_shutdown(async move {
            while !shutdown.load(Ordering::Relaxed) {
                sleep(Duration::from_secs(1)).await
            }
            info!("[Metrics] Server exited successfully")
        });

    if let Err(e) = server.await {
        error!("[Metrics] Exit with error: {}", e);
    }
}

pub static SHRED_RECEIVED: Lazy<IntCounter> =
    Lazy::new(|| register_int_counter!("shred_received", " ",).unwrap());

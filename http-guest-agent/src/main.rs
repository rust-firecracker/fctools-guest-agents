use axum::{routing::post, Json, Router};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto,
    service::TowerToHyperService,
};
use serde::{Deserialize, Serialize};
use tokio_vsock::{VsockAddr, VsockListener, VMADDR_CID_ANY};
use tower_service::Service;

#[derive(Deserialize)]
struct PingRequest {
    a: u32,
    b: u32,
}

#[derive(Serialize)]
struct PingResponse {
    c: u32,
}

async fn ping_route(Json(request): Json<PingRequest>) -> Json<PingResponse> {
    let response = PingResponse {
        c: request.a * request.b,
    };

    Json(response)
}

const VSOCK_PORT: u32 = 8000;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let vsock_addr = VsockAddr::new(VMADDR_CID_ANY, VSOCK_PORT);
    let mut listener = VsockListener::bind(vsock_addr).expect("Could not bind to vsock address");

    let router = Router::new().route("/ping", post(ping_route));
    let mut make_service = router.into_make_service();

    loop {
        let (stream, peer_addr) = listener
            .accept()
            .await
            .expect("Could not accept vsock connection");
        let tower_service = make_service
            .call(&stream)
            .await
            .expect("Could not call make service to produce axum router");
        println!("Accepted connection from: {peer_addr}");

        tokio::task::spawn(async move {
            let tokio_io = TokioIo::new(stream);
            let hyper_service = TowerToHyperService::new(tower_service);

            if let Err(err) = auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(tokio_io, hyper_service)
                .await
            {
                eprintln!("Failed serving connection: {err}");
            }
        });
    }
}

use std::pin::Pin;

use definitions::{
    guest_agent_service_server::{GuestAgentService, GuestAgentServiceServer},
    Ping, Pong,
};
use tokio_stream::Stream;
use tokio_vsock::{VsockAddr, VsockListener, VMADDR_CID_ANY};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use transport::VsockListenerStream;

mod transport;

mod definitions {
    tonic::include_proto!("guest_agent");
}

struct App;

#[tonic::async_trait]
impl GuestAgentService for App {
    async fn unary(&self, request: Request<Ping>) -> Result<Response<Pong>, Status> {
        Ok(Response::new(Pong {
            number: request.into_inner().number.pow(2),
        }))
    }

    async fn client_streaming(
        &self,
        request: Request<Streaming<Ping>>,
    ) -> Result<Response<Pong>, Status> {
        let mut streaming = request.into_inner();
        let mut product = 1;

        while let Ok(Some(message)) = streaming.message().await {
            if message.number != 0 {
                product *= message.number;
            }
        }

        Ok(Response::new(Pong { number: product }))
    }

    type ServerStreamingStream = Pin<Box<dyn Stream<Item = Result<Pong, Status>> + Send + 'static>>;

    async fn server_streaming(
        &self,
        request: Request<Ping>,
    ) -> Result<Response<Self::ServerStreamingStream>, Status> {
        todo!()
    }

    type DuplexStreamingStream = Pin<Box<dyn Stream<Item = Result<Pong, Status>> + Send + 'static>>;

    async fn duplex_streaming(
        &self,
        request: Request<Streaming<Ping>>,
    ) -> Result<Response<Self::DuplexStreamingStream>, Status> {
        todo!()
    }
}

const VSOCK_PORT: u32 = 9000;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let vsock_listener = VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, VSOCK_PORT))
        .expect("Could not bind vsock");
    let vsock_listener_stream = VsockListenerStream::new(vsock_listener);

    Server::builder()
        .add_service(GuestAgentServiceServer::new(App))
        .serve_with_incoming(vsock_listener_stream)
        .await
        .expect("Could not serve gRPC over vsock");
}

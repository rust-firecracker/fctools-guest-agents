use std::{pin::Pin, time::Duration};

use definitions::{
    guest_agent_service_server::{GuestAgentService, GuestAgentServiceServer},
    Ping, Pong,
};
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::{wrappers::UnboundedReceiverStream, Stream};
use tokio_vsock::{VsockAddr, VsockListener, VMADDR_CID_ANY};
use tonic::{transport::Server, Request, Response, Status, Streaming};

mod definitions {
    tonic::include_proto!("guest_agent");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("guest_agent_descriptor");
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
        let (stream, rx) = unbounded_channel();
        let request = request.into_inner();

        tokio::task::spawn(async move {
            for _ in 1..=request.number {
                stream
                    .send(Ok(Pong {
                        number: request.number,
                    }))
                    .expect("Could not respond with pong over stream");
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });

        Ok(Response::new(Box::pin(UnboundedReceiverStream::new(rx))))
    }

    type DuplexStreamingStream = Pin<Box<dyn Stream<Item = Result<Pong, Status>> + Send + 'static>>;

    async fn duplex_streaming(
        &self,
        request: Request<Streaming<Ping>>,
    ) -> Result<Response<Self::DuplexStreamingStream>, Status> {
        let mut recv_stream = request.into_inner();
        let (send_stream, rx) = unbounded_channel();

        tokio::task::spawn(async move {
            while let Ok(Some(ping)) = recv_stream.message().await {
                send_stream
                    .send(Ok(Pong {
                        number: ping.number,
                    }))
                    .expect("Could not respond with pong over stream");
            }
        });

        Ok(Response::new(Box::pin(UnboundedReceiverStream::new(rx))))
    }
}

const VSOCK_PORT: u32 = 9000;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let vsock_listener = VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, VSOCK_PORT))
        .expect("Could not bind vsock");
    let v1_reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(definitions::FILE_DESCRIPTOR_SET)
        .build_v1()
        .expect("Could not build v1 server reflection");

    Server::builder()
        .add_service(v1_reflection)
        .add_service(GuestAgentServiceServer::new(App))
        .serve_with_incoming(vsock_listener.incoming())
        .await
        .expect("Could not serve gRPC over vsock");
}

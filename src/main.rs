use crate::protocol::{Command, Error};
use log::{debug, error, info};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;

mod protocol;

const MAX_LENGTH: usize = 8 + 128;
const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let listener = net::TcpListener::bind("127.0.0.1:8181").await?;
    info!("started listening on '127.0.0.1:8181'");
    while let Ok((mut stream, incoming_socket_addr)) = listener.accept().await {
        debug!("incoming connection from {:?}", incoming_socket_addr);

        let mut buffer = [0; MAX_LENGTH];
        let read = stream.read(&mut buffer).await?;
        let Ok(request) = protocol::Request::from_bytes(&buffer[..read]).map_err(|err| {
            error!("could not parse the request {:?}", err);
        }) else {
            continue;
        };

        debug!("request = {:?}", request);

        match request.command {
            Command::Connect => {
                if let Err(err) = handle_connect(stream, request).await {
                    error!("{:?}", err);
                }
            }
            Command::Bind => {
                todo!("unsupported command")
            }
        }
    }
    Ok(())
}

async fn handle_connect(
    mut stream: net::TcpStream,
    request: protocol::Request,
) -> Result<(), Error> {
    let addr = SocketAddrV4::new(Ipv4Addr::from(request.dest_ip), request.dest_port);

    debug!("connecting to {:?}", addr);

    let upstream =
        tokio::time::timeout(DEFAULT_CONNECTION_TIMEOUT, net::TcpStream::connect(addr)).await;

    match upstream {
        Ok(Ok(upstream)) => {
            stream
                .write_all(&protocol::Response::default().to_bytes())
                .await?;
            tokio::spawn(proxy_connection(stream, upstream));

            Ok(())
        }
        Err(e) => {
            stream
                .write_all(&protocol::Response::reject_response().to_bytes())
                .await?;
            Err(e)?
        }
        Ok(Err(e)) => {
            stream
                .write_all(&protocol::Response::reject_response().to_bytes())
                .await?;
            Err(e)?
        }
    }
}

async fn proxy_connection(mut incoming: net::TcpStream, mut upstream: net::TcpStream) {
    let (mut read_incoming, mut write_incoming) = incoming.split();
    let (mut read_upstream, mut write_upstream) = upstream.split();

    let _ = tokio::join!(
        tokio::io::copy(&mut read_incoming, &mut write_upstream),
        tokio::io::copy(&mut read_upstream, &mut write_incoming)
    );
}

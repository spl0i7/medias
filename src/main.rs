use crate::protocol::{Command, Error, Reply};
use log::{debug, error, info};
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;

mod protocol;

const MAX_LENGTH: usize = 8 + 128;

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
    let response = protocol::Response {
        version: 0,
        reply: Reply::RequestGranted,
        dest_port: 0,
        dest_ip: 0,
    };

    stream.write_all(&response.to_bytes()).await?;

    let ip_bytes = request.dest_ip.to_be_bytes();

    let addr = SocketAddrV4::new(
        Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]),
        request.dest_port,
    );

    debug!("connecting to {:?}", addr);
    let upstream = net::TcpStream::connect(addr).await?;
    tokio::spawn(async move {
        proxy_connection(stream, upstream).await;
    });

    Ok(())
}

async fn proxy_connection(mut incoming: net::TcpStream, mut upstream: net::TcpStream) {
    let (mut read_incoming, mut write_incoming) = incoming.split();
    let (mut read_upstream, mut write_upstream) = upstream.split();

    tokio::select! {
        _ = tokio::io::copy(&mut read_incoming, &mut write_upstream) => {}
        _ = tokio::io::copy(&mut read_upstream, &mut write_incoming) => {}
    }
}

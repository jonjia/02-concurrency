use std::{io, net::SocketAddr};

use anyhow::Result;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tracing::{info, warn};

const BUF_SIZE: usize = 4096;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:6379";

    let listener = TcpListener::bind(addr).await?;
    info!("Dredis: Listening on: {}", addr);

    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Dredis: Accepted connection from: {}", raddr);
        tokio::spawn(async move {
            if let Err(e) = process_redis_connect(stream, raddr).await {
                warn!(
                    "Dredis: Error processing connection with {}: {:?}",
                    raddr, e
                );
            }
        });
    }
}

async fn process_redis_connect(mut stream: TcpStream, raddr: SocketAddr) -> Result<()> {
    loop {
        stream.readable().await?;
        let mut buf = Vec::with_capacity(BUF_SIZE);

        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                info!("Dredis: Read {} bytes", n);
                let line = String::from_utf8_lossy(&buf);
                info!("Dredis: Line: {:?}", line);
                stream.write_all(b"+OK\r\n").await?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    warn!("Dredis: Connection closed on {}", raddr);
    Ok(())
}

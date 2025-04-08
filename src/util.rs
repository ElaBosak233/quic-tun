use std::net::SocketAddr;

use tokio::io::AsyncWriteExt;
use tracing::error;

pub async fn parse_addr(addr: String) -> Result<SocketAddr, anyhow::Error> {
    match tokio::net::lookup_host(&addr).await {
        Ok(mut addr) => Ok(addr.next().unwrap()),
        Err(err) => {
            error!("Failed to resolve destination: {:?}", err);
            std::process::exit(1);
        }
    }
}

pub async fn bidirectional_copy(
    tcp_stream: &mut tokio::net::TcpStream, mut quic_stream: (quinn::SendStream, quinn::RecvStream),
) -> Result<(), anyhow::Error> {
    let (mut quic_send, mut quic_recv) = quic_stream;

    let (mut tcp_read, mut tcp_write) = tcp_stream.split();

    let tcp_to_quic = async {
        tokio::io::copy(&mut tcp_read, &mut quic_send).await?;
        quic_send.finish()?;
        Ok::<(), anyhow::Error>(())
    };

    let quic_to_tcp = async {
        tokio::io::copy(&mut quic_recv, &mut tcp_write).await?;
        tcp_write.shutdown().await?;
        Ok::<(), anyhow::Error>(())
    };

    tokio::try_join!(tcp_to_quic, quic_to_tcp)?;

    tcp_stream.shutdown().await?;
    Ok(())
}

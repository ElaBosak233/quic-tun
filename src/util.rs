use tokio::io::AsyncWriteExt;

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

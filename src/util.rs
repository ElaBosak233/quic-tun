use tokio::io::AsyncWriteExt;

pub async fn bidirectional_copy(
    tcp_stream: &mut tokio::net::TcpStream,
    mut send_stream: quinn::SendStream,
    mut recv_stream: quinn::RecvStream,
) -> Result<(), anyhow::Error> {
    let (mut tcp_reader, mut tcp_writer) = tcp_stream.split();

    let tcp_to_quic = async {
        let result = tokio::io::copy(&mut tcp_reader, &mut send_stream).await;
        send_stream.finish()?;
        result
    };

    let quic_to_tcp = async {
        let result = tokio::io::copy(&mut recv_stream, &mut tcp_writer).await;
        tcp_writer.shutdown().await?;
        result
    };

    tokio::select! {
        res = tcp_to_quic => {
            res?;
            recv_stream.stop(0u32.into()).ok();
        }
        res = quic_to_tcp => {
            res?;
            send_stream.finish().ok();
        }
    }

    tcp_stream.shutdown().await?;
    Ok(())
}
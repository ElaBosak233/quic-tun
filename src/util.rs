pub async fn bidirectional_copy(
    tcp_stream: &mut tokio::net::TcpStream, mut send_stream: quinn::SendStream,
    mut recv_stream: quinn::RecvStream,
) -> Result<(), anyhow::Error> {
    let (mut tcp_reader, mut tcp_writer) = tcp_stream.split();

    let tcp_to_quic = async {
        let bytes_copied = tokio::io::copy(&mut tcp_reader, &mut send_stream).await?;
        send_stream.finish()?;
        Ok(bytes_copied)
    };

    let quic_to_tcp = tokio::io::copy(&mut recv_stream, &mut tcp_writer);

    tokio::try_join!(tcp_to_quic, quic_to_tcp)?;
    Ok(())
}

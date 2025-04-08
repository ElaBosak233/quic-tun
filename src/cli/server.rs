use std::{net::SocketAddr, sync::Arc};
use std::path::PathBuf;
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use tokio::net::TcpStream;
use tracing::{error, info};

pub async fn launch(dest: SocketAddr, bind: SocketAddr, cert: Option<PathBuf>, key: Option<PathBuf>) -> Result<(), anyhow::Error> {
    let (endpoint, _server_cert) = make_server_endpoint(bind, cert, key)?;
    info!("Listening on {}", bind);

    loop {
        let incoming = endpoint.accept().await.unwrap();

        tokio::spawn(async move {
            let quic_conn = match incoming.await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("QUIC connection failed: {}", e);
                    return;
                }
            };

            info!(
                "QUIC connection established: addr={}",
                quic_conn.remote_address()
            );

            loop {
                match quic_conn.accept_bi().await {
                    Ok(quic_stream) => {
                        let target = dest.clone();
                        tokio::spawn(async move {
                            match TcpStream::connect(target).await {
                                Ok(mut tcp_stream) => {
                                    if let Err(e) = crate::util::bidirectional_copy(
                                        &mut tcp_stream,
                                        quic_stream,
                                    )
                                    .await
                                    {
                                        error!("Copy error: {}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("TCP connect failed: {}", e);
                                }
                            }
                        });
                    }
                    Err(quinn::ConnectionError::TimedOut) => {
                        info!("QUIC connection timed out");
                        break;
                    }
                    Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                        info!("QUIC connection closed");
                        break;
                    }
                    Err(e) => {
                        error!("Stream accept error: {}", e);
                        break;
                    }
                }
            }
        });
    }
}

pub fn make_server_endpoint(
    bind_addr: SocketAddr, cert: Option<PathBuf>, key: Option<PathBuf>
) -> Result<(Endpoint, CertificateDer<'static>), anyhow::Error> {
    let (server_config, server_cert) = configure_server(cert, key)?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, server_cert))
}

fn configure_server(cert: Option<PathBuf>, key: Option<PathBuf>) -> Result<(ServerConfig, CertificateDer<'static>), anyhow::Error> {
    let (cert_der, key_der) = if let (Some(cert_path), Some(key_path)) = (cert, key) {
        // Load cert/key from file
        let cert_bytes = std::fs::read(&cert_path)?;
        let key_bytes = std::fs::read(&key_path)?;

        let cert = CertificateDer::from(cert_bytes);
        let key = PrivatePkcs8KeyDer::from(key_bytes);

        (cert, key)
    } else {
        // Fallback to self-signed
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
        let cert_der = CertificateDer::from(cert.cert);
        let key_der = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
        (cert_der, key_der)
    };

    let mut server_config = ServerConfig::with_single_cert(vec![cert_der.clone()], key_der.into())?;
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config
        .max_concurrent_bidi_streams(100_u32.into())
        .max_concurrent_uni_streams(0_u8.into())
        .keep_alive_interval(Some(std::time::Duration::from_secs(2)));

    Ok((server_config, cert_der))
}

use std::{net::SocketAddr, sync::Arc};

use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use tokio::net::TcpStream;
use tracing::{error, info};

pub async fn launch(target: String, bind: SocketAddr) -> Result<(), anyhow::Error> {
    let (endpoint, _server_cert) = make_server_endpoint(bind)?;

    loop {
        let incoming = endpoint.accept().await.unwrap();
        let target = target.clone();

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
                    Ok((send_stream, recv_stream)) => {
                        let target = target.clone();
                        tokio::spawn(async move {
                            match TcpStream::connect(target).await {
                                Ok(mut tcp) => {
                                    if let Err(e) = crate::util::bidirectional_copy(
                                        &mut tcp,
                                        send_stream,
                                        recv_stream,
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
    bind_addr: SocketAddr,
) -> Result<(Endpoint, CertificateDer<'static>), anyhow::Error> {
    let (server_config, server_cert) = configure_server()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, server_cert))
}

fn configure_server() -> Result<(ServerConfig, CertificateDer<'static>), anyhow::Error> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());

    let mut server_config =
        ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into())?;
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    Ok((server_config, cert_der))
}

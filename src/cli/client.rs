use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};

use quinn::{ClientConfig, Endpoint, crypto::rustls::QuicClientConfig};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use tokio::net::TcpListener;
use tracing::{error, info};

pub async fn launch(
    dest: SocketAddr, bind: SocketAddr, cert: Option<PathBuf>, insecure: bool,
) -> Result<(), anyhow::Error> {
    let mut roots = Vec::new();
    if let Some(ca_path) = cert {
        roots.push(fs::read(ca_path)?);
    }
    let roots: &[&[u8]] = &roots.iter().map(|v| v.as_slice()).collect::<Vec<_>>();

    let listener = TcpListener::bind(bind).await?;
    let endpoint = make_client_endpoint(bind, roots, insecure)?;
    info!("Listening on {}", bind);

    loop {
        let (mut tcp_stream, _) = listener.accept().await?;
        let connection = endpoint.connect(dest, "server")?.await?;
        info!("Connected: addr={}", connection.remote_address());

        tokio::spawn(async move {
            let quic_stream = match connection.open_bi().await {
                Ok(streams) => streams,
                Err(e) => {
                    error!("Failed to open stream: {}", e);
                    return;
                }
            };

            if let Err(e) = crate::util::bidirectional_copy(&mut tcp_stream, quic_stream).await {
                error!("Copy error: {}", e);
            }
        });
    }
}

pub fn make_client_endpoint(
    bind_addr: SocketAddr, root_certs: &[&[u8]], insecure: bool,
) -> Result<Endpoint, anyhow::Error> {
    let mut endpoint = Endpoint::client(bind_addr)?;
    if insecure {
        endpoint.set_default_client_config(ClientConfig::new(Arc::new(
            QuicClientConfig::try_from(
                rustls::ClientConfig::builder()
                    .dangerous()
                    .with_custom_certificate_verifier(SkipServerVerification::new())
                    .with_no_client_auth(),
            )?,
        )));
    } else {
        let client_cfg = configure_client(root_certs)?;
        endpoint.set_default_client_config(client_cfg);
    }
    Ok(endpoint)
}

fn configure_client(root_certs: &[&[u8]]) -> Result<ClientConfig, anyhow::Error> {
    let mut certs = rustls::RootCertStore::empty();
    for cert in root_certs {
        certs.add(CertificateDer::from(*cert))?;
    }

    let mut transport = quinn::TransportConfig::default();
    transport.keep_alive_interval(Some(std::time::Duration::from_secs(2)));

    let mut client_config = ClientConfig::with_root_certificates(Arc::new(certs))?;
    client_config.transport_config(Arc::new(transport));

    Ok(client_config)
}

#[derive(Debug)]
struct SkipServerVerification(Arc<rustls::crypto::CryptoProvider>);

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self(Arc::new(rustls::crypto::ring::default_provider())))
    }
}

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self, _end_entity: &CertificateDer<'_>, _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>, _ocsp: &[u8], _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self, message: &[u8], cert: &CertificateDer<'_>, dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self, message: &[u8], cert: &CertificateDer<'_>, dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

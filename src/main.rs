mod cli;
mod util;

use std::{net::SocketAddr, path::PathBuf};

use clap::{Parser, command};
use tracing::error;

#[derive(Parser)]
#[command(name = "quic-tun", bin_name = "quic-tun", version, about)]
enum QuicTunCli {
    /// Connect to a QUIC tunnel server and forward traffic (client mode)
    #[clap(alias("c"))]
    Connect {
        /// Destination address to connect to (domain:port or IP:port)
        #[clap(long)]
        dest: String,

        /// Local address to bind to (default: 0.0.0.0:4443)
        #[clap(long, default_value = "0.0.0.0:4443")]
        bind: SocketAddr,

        /// Path to custom certificate authority (CA) file (optional)
        #[clap(long)]
        cert: Option<PathBuf>,

        /// Skip certificate verification (dangerous, for testing only)
        #[clap(long, default_value = "false")]
        insecure: bool,
    },

    /// Run as a QUIC tunnel server and forward traffic to destination
    #[clap(alias("s"))]
    Serve {
        /// Destination address to forward to (IP/domain:port)
        #[clap(long)]
        dest: String,

        /// Address to listen for incoming QUIC connections (default: 0.0.0.0:4443)
        #[clap(long, default_value = "0.0.0.0:4443")]
        bind: SocketAddr,

        /// Path to TLS certificate in PEM or DER format (optional)
        #[clap(long)]
        cert: Option<PathBuf>,

        /// Path to private key in PEM or DER format (optional)
        #[clap(long)]
        key: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    cli::logger::init();
    rustls::crypto::ring::default_provider()
        .install_default()
        .inspect_err(|err| {
            error!("Failed to install default crypto provider: {:?}", err);
            std::process::exit(1);
        })
        .ok();

    let quic_tun_cli = QuicTunCli::parse();

    match quic_tun_cli {
        QuicTunCli::Connect {
            dest,
            bind,
            cert,
            insecure,
        } => {
            cli::client::launch(util::parse_addr(dest).await?, bind, cert, insecure).await?;
        }
        QuicTunCli::Serve { dest, bind, cert, key } => {
            cli::server::launch(util::parse_addr(dest).await?, bind, cert, key).await?;
        }
    }

    Ok(())
}

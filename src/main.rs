mod cli;
mod util;

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, command};
use tracing::error;

#[derive(Parser)]
#[command(name = "quic-tun", bin_name = "quic-tun", version, about)]
enum QuicTunCli {
    #[clap(alias("c"))]
    Connect {
        dest: SocketAddr,
        bind: SocketAddr,
        #[clap(long)]
        cert: Option<PathBuf>,
        #[clap(long, default_value = "false")]
        insecure: bool,
    },
    #[clap(alias("s"))]
    Serve { dest: SocketAddr, bind: SocketAddr },
}

#[tokio::main]
async fn main() {
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
            cli::client::launch(dest, bind, cert, insecure)
                .await
                .expect("");
        }
        QuicTunCli::Serve { dest, bind } => {
            cli::server::launch(dest, bind).await.expect("");
        }
    }
}

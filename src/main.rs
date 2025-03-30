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
        server: String,
        bind: SocketAddr,
        #[clap(long = "ca")]
        ca: Option<PathBuf>,
        #[clap(long = "insecure", default_value = "false")]
        insecure: bool,
    },
    #[clap(alias("s"))]
    Serve { target: String, bind: SocketAddr },
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
            server,
            bind,
            ca,
            insecure,
        } => {
            cli::client::launch(server, bind, ca, insecure)
                .await
                .expect("");
        }
        QuicTunCli::Serve { target, bind } => {
            cli::server::launch(target, bind).await.expect("");
        }
    }
}

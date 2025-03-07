mod cli;
mod util;

use std::path::PathBuf;

use clap::{Parser, command};
use tracing::error;

#[derive(Parser)]
#[command(name = "quic-tun", bin_name = "quic-tun", version, about)]
enum QuicTunCli {
    #[clap(alias("c"))]
    Connect {
        server: String,
        port: u16,
        #[clap(long = "ca")]
        ca: Option<PathBuf>,
        #[clap(long = "insecure", default_value = "false")]
        insecure: bool,
    },
    #[clap(alias("s"))]
    Serve { target: String, port: u16 },
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
            port,
            ca,
            insecure,
        } => {
            cli::client::launch(server, port, ca, insecure)
                .await
                .expect("");
        }
        QuicTunCli::Serve { target, port } => {
            cli::server::launch(target, port).await.expect("");
        }
    }
}

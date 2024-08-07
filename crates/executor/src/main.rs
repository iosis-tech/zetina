pub mod executor;

use axum::Router;
use clap::Parser;
use executor::Executor;
use libp2p::Multiaddr;
use starknet::{core::types::FieldElement, signers::SigningKey};
use std::{str::FromStr, time::Duration};
use tokio::{net::TcpListener, sync::mpsc};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;
use zetina_common::graceful_shutdown::shutdown_signal;
use zetina_peer::swarm::{GossipsubMessage, SwarmRunner};
use zetina_prover::stone_prover::StoneProver;
use zetina_runner::cairo_runner::CairoRunner;

#[derive(Parser)]
struct Cli {
    /// The private key as a hex string
    #[arg(short, long)]
    private_key: String,

    #[arg(short, long)]
    address: String,

    #[arg(short, long)]
    dial_addresses: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    // Parse command line arguments
    let cli = Cli::parse();

    // TODO: common setup in node initiate binary
    let private_key = hex::decode(cli.private_key)?;
    let secret_key = libp2p::identity::ecdsa::SecretKey::try_from_bytes(private_key.as_slice())?;
    let p2p_keypair =
        libp2p::identity::Keypair::from(libp2p::identity::ecdsa::Keypair::from(secret_key));

    let identity = p2p_keypair.public().to_peer_id();

    let signing_key = SigningKey::from_secret_scalar(
        FieldElement::from_byte_slice_be(private_key.as_slice()).unwrap(),
    );

    let ws_root = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"),
    )
    .join("../../");
    let bootloader_program_path = ws_root.join("target/bootloader.json");

    let mut swarm_runner =
        SwarmRunner::new(p2p_keypair, Multiaddr::from_str(&cli.address).unwrap())?;

    cli.dial_addresses
        .into_iter()
        .try_for_each(|addr| swarm_runner.swarm.dial(Multiaddr::from_str(&addr).unwrap()))
        .unwrap();

    let (gossipsub_tx, gossipsub_rx) = mpsc::channel::<GossipsubMessage>(100);
    let swarm_events = swarm_runner.run(gossipsub_rx);

    let runner = CairoRunner::new(bootloader_program_path, signing_key.verifying_key());
    let prover = StoneProver::new();

    Executor::new(identity, swarm_events, gossipsub_tx, runner, prover);

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind("0.0.0.0:3010").await.unwrap();

    // Run the server with graceful shutdown
    axum::serve(
        listener,
        Router::new().layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
        )),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}

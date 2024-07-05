pub mod executor;
pub mod swarm;

use axum::Router;
use clap::Parser;
use executor::Executor;
use libp2p::gossipsub;
use std::time::Duration;
use swarm::SwarmRunner;
use tokio::{net::TcpListener, sync::mpsc};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;
use zetina_common::{
    graceful_shutdown::shutdown_signal,
    network::Network,
    node_account::NodeAccount,
    topic::{gossipsub_ident_topic, Topic},
};
use zetina_prover::stone_prover::StoneProver;
use zetina_runner::cairo_runner::CairoRunner;

#[derive(Parser)]
struct Cli {
    /// The private key as a hex string
    #[arg(short, long)]
    private_key: String,

    #[arg(short, long)]
    dial_addresses: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    // Parse command line arguments
    let cli = Cli::parse();

    let ws_root = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"),
    )
    .join("../../");
    let bootloader_program_path = ws_root.join("target/bootloader.json");

    // TODO: common setup in node initiate binary
    let network = Network::Sepolia;
    let private_key = hex::decode(cli.private_key)?;

    let node_account = NodeAccount::new(private_key);

    // Generate topic
    let new_job_topic = gossipsub_ident_topic(network, Topic::NewJob);
    let picked_job_topic = gossipsub_ident_topic(network, Topic::PickedJob);
    let finished_job_topic = gossipsub_ident_topic(network, Topic::FinishedJob);

    let (swarm_events_tx, swarm_events_rx) = mpsc::channel::<gossipsub::Event>(100);
    let (picked_job_topic_tx, picked_job_topic_rx) = mpsc::channel::<Vec<u8>>(1000);
    let (finished_job_topic_tx, finished_job_topic_rx) = mpsc::channel::<Vec<u8>>(1000);

    SwarmRunner::new(
        cli.dial_addresses,
        node_account.get_keypair(),
        vec![new_job_topic, picked_job_topic.to_owned(), finished_job_topic.to_owned()],
        vec![
            (picked_job_topic.to_owned(), picked_job_topic_rx),
            (finished_job_topic, finished_job_topic_rx),
        ],
        swarm_events_tx,
    )?;

    let verifying_key = node_account.get_verifying_key();
    let runner = CairoRunner::new(bootloader_program_path, verifying_key);
    let prover = StoneProver::new();

    Executor::new(swarm_events_rx, finished_job_topic_tx, picked_job_topic_tx, runner, prover);

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

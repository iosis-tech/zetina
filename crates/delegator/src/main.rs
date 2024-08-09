pub mod api;
pub mod bid_queue;
pub mod delegator;

use api::ServerState;
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use clap::Parser;
use delegator::Delegator;
use libp2p::Multiaddr;
use starknet::{core::types::FieldElement, signers::SigningKey};
use std::{str::FromStr, time::Duration};
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc},
};
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;
use zetina_common::{graceful_shutdown::shutdown_signal, job::JobData, job_witness::JobWitness};
use zetina_peer::swarm::{GossipsubMessage, SwarmRunner};

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

    let signing_key = SigningKey::from_secret_scalar(
        FieldElement::from_byte_slice_be(private_key.as_slice()).unwrap(),
    );

    let mut swarm_runner =
        SwarmRunner::new(p2p_keypair, Multiaddr::from_str(&cli.address).unwrap())?;

    cli.dial_addresses
        .into_iter()
        .try_for_each(|addr| swarm_runner.swarm.dial(Multiaddr::from_str(&addr).unwrap()))
        .unwrap();

    let (gossipsub_tx, gossipsub_rx) = mpsc::channel::<GossipsubMessage>(100);
    let (delegate_tx, delegate_rx) = mpsc::channel::<JobData>(100);
    let (finished_tx, finished_rx) = broadcast::channel::<JobWitness>(100);
    let swarm_events = swarm_runner.run(gossipsub_rx);

    Delegator::new(swarm_events, gossipsub_tx, delegate_rx, finished_tx, signing_key);

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    // Run the server with graceful shutdown
    axum::serve(
        listener,
        Router::new()
            .route("/delegate", post(api::deletage_handler))
            .route("/job_events", get(api::job_events_handler))
            .layer((
                TraceLayer::new_for_http(),
                // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                // requests don't hang forever.
                TimeoutLayer::new(Duration::from_secs(10)),
                CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any),
                DefaultBodyLimit::disable(),
            ))
            .with_state(ServerState { delegate_tx, finished_rx }),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}

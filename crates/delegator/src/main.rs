pub mod api;
pub mod delegator;
pub mod swarm;

use api::ServerState;
use axum::{
    extract::DefaultBodyLimit, routing::{get, post}, Router
};
use clap::Parser;
use delegator::Delegator;
use libp2p::gossipsub;
use std::time::Duration;
use swarm::SwarmRunner;
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
use zetina_common::{
    graceful_shutdown::shutdown_signal,
    job::Job,
    job_witness::JobWitness,
    network::Network,
    node_account::NodeAccount,
    topic::{gossipsub_ident_topic, Topic},
};

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

    // TODO: common setup in node initiate binary
    let network = Network::Sepolia;
    let private_key = hex::decode(cli.private_key)?;

    let node_account = NodeAccount::new(private_key);

    // Generate topic
    let job_topic = gossipsub_ident_topic(network, Topic::NewJob);
    let picked_job_topic = gossipsub_ident_topic(network, Topic::PickedJob);
    let finished_job_topic = gossipsub_ident_topic(network, Topic::FinishedJob);

    let (swarm_events_tx, swarm_events_rx) = mpsc::channel::<gossipsub::Event>(100);
    let (job_picked_tx, job_picked_rx) = broadcast::channel::<Job>(100);
    let (job_witness_tx, job_witness_rx) = broadcast::channel::<JobWitness>(100);

    let (job_topic_tx, job_topic_rx) = mpsc::channel::<Vec<u8>>(100);

    SwarmRunner::new(
        cli.dial_addresses,
        node_account.get_keypair(),
        vec![job_topic.to_owned(), picked_job_topic, finished_job_topic],
        vec![(job_topic, job_topic_rx)],
        swarm_events_tx,
    )?;

    Delegator::new(job_picked_tx, job_witness_tx, swarm_events_rx);

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind("0.0.0.0:3010").await.unwrap();

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
                DefaultBodyLimit::disable()
            ))
            .with_state(ServerState {
                signing_key: node_account.get_signing_key().to_owned(),
                job_topic_tx,
                job_picked_rx,
                job_witness_rx,
            }),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}

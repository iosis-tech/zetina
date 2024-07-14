pub mod api;
pub mod delegator;
pub mod swarm;

use api::ServerState;
use axum::{
    routing::{get, post},
    Router,
};
use delegator::Delegator;
use libp2p::gossipsub;
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient, Url};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    // TODO: common setup in node initiate binary
    let network = Network::Sepolia;
    let private_key =
        hex::decode("018ef9563461ec2d88236d59039babf44c97d8bf6200d01d81170f1f60a78f32")?;
    let account_address =
        hex::decode("cdd51fbc4e008f4ef807eaf26f5043521ef5931bbb1e04032a25bd845d286b")?;
    let url = "https://starknet-sepolia.public.blastapi.io";

    let node_account = NodeAccount::new(
        private_key,
        account_address,
        network,
        JsonRpcClient::new(HttpTransport::new(Url::parse(url)?)),
    );

    // Generate topic
    let job_topic = gossipsub_ident_topic(network, Topic::NewJob);
    let picked_job_topic = gossipsub_ident_topic(network, Topic::PickedJob);
    let finished_job_topic = gossipsub_ident_topic(network, Topic::FinishedJob);

    let (swarm_events_tx, swarm_events_rx) = mpsc::channel::<gossipsub::Event>(100);
    let (job_picked_tx, job_picked_rx) = broadcast::channel::<Job>(100);
    let (job_witness_tx, job_witness_rx) = broadcast::channel::<JobWitness>(100);

    let (job_topic_tx, job_topic_rx) = mpsc::channel::<Vec<u8>>(100);

    SwarmRunner::new(
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

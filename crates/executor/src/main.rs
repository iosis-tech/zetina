use axum::Router;
use clap::Parser;
use libp2p::{
    gossipsub::{self},
    Multiaddr,
};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::{str::FromStr, time::Duration};
use tokio::{net::TcpListener, sync::mpsc};
use tokio_stream::StreamExt;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing::error;
use tracing_subscriber::EnvFilter;
use zetina_common::{graceful_shutdown::shutdown_signal, hash, job::JobBid};
use zetina_peer::swarm::{
    DelegationMessage, GossipsubMessage, MarketMessage, PeerBehaviourEvent, SwarmRunner, Topic,
};

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

    let mut swarm_runner =
        SwarmRunner::new(p2p_keypair, Multiaddr::from_str(&cli.address).unwrap())?;

    cli.dial_addresses
        .into_iter()
        .try_for_each(|addr| swarm_runner.swarm.dial(Multiaddr::from_str(&addr).unwrap()))
        .unwrap();

    let (gossipsub_tx, gossipsub_rx) = mpsc::channel::<GossipsubMessage>(100);
    let mut swarm_events = swarm_runner.run(gossipsub_rx);

    tokio::spawn(async move {
        while let Some(event) = swarm_events.next().await {
            match event {
                PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. }) => {
                    if message.topic == Topic::Market.into() {
                        match serde_json::from_slice::<MarketMessage>(&message.data) {
                            Ok(MarketMessage::Job(job)) => {
                                gossipsub_tx
                                    .send(GossipsubMessage {
                                        topic: Topic::Market.into(),
                                        data: serde_json::to_vec(&MarketMessage::JobBid(JobBid {
                                            job_hash: hash!(job),
                                            price: 1000,
                                        }))
                                        .unwrap(),
                                    })
                                    .await
                                    .unwrap();
                            }
                            Err(error) => {
                                error! {"Deserialization error: {:?}", error};
                            }
                            _ => {}
                        }
                    }
                    if message.topic == Topic::Delegation.into() {
                        match serde_json::from_slice::<DelegationMessage>(&message.data) {
                            Ok(DelegationMessage::Delegate(_job)) => {
                                //TODO run and prove
                            }
                            Err(error) => {
                                error! {"Deserialization error: {:?}", error};
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    });

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

use futures_util::StreamExt;
use sharp_p2p_common::network::Network;
use sharp_p2p_common::topic::{gossipsub_ident_topic, Topic};
use sharp_p2p_peer::registry::RegistryHandler;
use sharp_p2p_peer::swarm::SwarmRunner;
use std::error::Error;
use tokio::sync::mpsc;
use tracing::debug;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    // 1. Generate keypair for the node
    let p2p_local_keypair = libp2p::identity::Keypair::generate_ed25519();

    // 2. Generate topic
    let topic = gossipsub_ident_topic(Network::Sepolia, Topic::NewJob);

    let swarm_runner = SwarmRunner::new(&p2p_local_keypair, &topic)?;
    let mut registry_handler = RegistryHandler::new(
        "https://starknet-sepolia.public.blastapi.io",
        "0xcdd51fbc4e008f4ef807eaf26f5043521ef5931bbb1e04032a25bd845d286b",
    );

    let (_send_topic_tx, send_topic_rx) = mpsc::channel::<Vec<u8>>(1000);
    let mut message_stream = swarm_runner.run(topic, send_topic_rx);
    let mut event_stream = registry_handler.subscribe_events(vec!["0x0".to_string()]);

    loop {
        tokio::select! {
            Some(event) = message_stream.next() => {
                debug!("{:?}", event);
            },
            Some(Ok(event_vec)) = event_stream.next() => {
                debug!("{:?}", event_vec);
            },
        }
    }
}

#![deny(unused_crate_dependencies)]

use futures::StreamExt;
use libp2p::gossipsub::Event;
use sharp_p2p_common::{
    hash,
    identity::IdentityHandler,
    job::Job,
    network::Network,
    topic::{gossipsub_ident_topic, Topic},
};
use sharp_p2p_compiler::{
    cairo_compiler::tests::models::fixture, cairo_compiler::CairoCompiler,
    traits::CompilerController,
};
use sharp_p2p_peer::{registry::RegistryHandler, swarm::SwarmRunner};
use std::hash::{DefaultHasher, Hash, Hasher};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    sync::mpsc,
};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    let private_key =
        hex::decode("018ef9563461ec2d88236d59039babf44c97d8bf6200d01d81170f1f60a78f32")?;
    let identity_handler = IdentityHandler::new(private_key);
    let p2p_local_keypair = identity_handler.get_keypair();

    // Generate topic
    let new_job_topic = gossipsub_ident_topic(Network::Sepolia, Topic::NewJob);
    let picked_job_topic = gossipsub_ident_topic(Network::Sepolia, Topic::PickedJob);

    let mut swarm_runner =
        SwarmRunner::new(&p2p_local_keypair, &[new_job_topic.to_owned(), picked_job_topic])?;
    let mut registry_handler = RegistryHandler::new(
        "https://starknet-sepolia.public.blastapi.io",
        "0xcdd51fbc4e008f4ef807eaf26f5043521ef5931bbb1e04032a25bd845d286b",
    );
    let registry_address = registry_handler.get_registry_address();

    let (send_topic_tx, send_topic_rx) = mpsc::channel::<Vec<u8>>(1000);
    let mut message_stream = swarm_runner.run(new_job_topic, send_topic_rx);
    let mut event_stream = registry_handler.subscribe_events(vec!["0x0".to_string()]);

    let p2p_local_keypair_ecdsa = p2p_local_keypair.try_into_ecdsa().unwrap();
    let compiler = CairoCompiler::new(&p2p_local_keypair_ecdsa, registry_address);

    // Read cairo program path from stdin
    let mut stdin = BufReader::new(stdin()).lines();

    loop {
        tokio::select! {
            Ok(Some(_)) = stdin.next_line() => {
                // TODO: handle fixture better way
                let fixture = fixture();
                let job = compiler.run(fixture.program_path, fixture.program_input_path).unwrap().await.unwrap();
                let serialized_job = serde_json::to_string(&job).unwrap();
                send_topic_tx.send(serialized_job.into()).await?;
                info!("Sent a new job: {}", hash!(&job));
            },
            Some(event) = message_stream.next() => {
                match event {
                    Event::Message { message, .. } => {
                        // Received a new-job message from the network
                        if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::NewJob).into() {
                            let job: Job = serde_json::from_slice(&message.data).unwrap();
                            info!("Received a new job event: {}", hash!(&job));

                        }
                        // Received a picked-job message from the network
                        if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::PickedJob).into() {
                            let job: Job = serde_json::from_slice(&message.data).unwrap();
                            info!("Received picked job event: {}", hash!(&job));
                        }
                    },
                    Event::Subscribed { peer_id, topic } => {
                        info!("{} subscribed to the topic {}", peer_id.to_string(), topic.to_string());
                    },
                    Event::Unsubscribed { peer_id, topic }=> {
                        info!("{} unsubscribed to the topic {}", peer_id.to_string(), topic.to_string());
                    },
                    _ => {}
                }
            },
            Some(Ok(event_vec)) = event_stream.next() => {
                debug!("{:?}", event_vec);
            },
            else => break
        }
    }

    Ok(())
}

use libp2p::gossipsub::Event;
use std::hash::{DefaultHasher, Hash, Hasher};
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::info;
use zetina_common::{
    hash,
    job::Job,
    job_witness::JobWitness,
    network::Network,
    topic::{gossipsub_ident_topic, Topic},
};

pub async fn delegator_loop(
    mut message_stream: mpsc::Receiver<Event>,
    job_witness_tx: broadcast::Sender<JobWitness>,
    cancellation_token: CancellationToken,
) {
    loop {
        tokio::select! {
            Some(event) = message_stream.recv() => {
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
                        // Received a finished-job message from the network
                        if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::FinishedJob).into() {
                            let job_witness: JobWitness = serde_json::from_slice(&message.data).unwrap();
                            info!("Received finished job event: {}", hash!(&job_witness));
                            job_witness_tx.send(job_witness).unwrap();
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
            _ = cancellation_token.cancelled() => {
                break
            }
            else => break
        }
    }
}

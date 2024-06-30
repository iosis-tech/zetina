use libp2p::gossipsub::Event;
use std::hash::{DefaultHasher, Hash, Hasher};
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tracing::info;
use zetina_common::{
    graceful_shutdown::shutdown_signal,
    hash,
    job::Job,
    job_witness::JobWitness,
    network::Network,
    topic::{gossipsub_ident_topic, Topic},
};

pub struct Delegator {
    handle: Option<JoinHandle<Result<(), DelegatorError>>>,
}

impl Delegator {
    pub fn new(
        job_witness_tx: broadcast::Sender<JobWitness>,
        mut events_rx: mpsc::Receiver<Event>,
    ) -> Self {
        Self {
            handle: Some(tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(event) = events_rx.recv() => {
                            match event {
                                Event::Message { message, .. } => {
                                    // Received a new-job message from the network
                                    if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::NewJob).into() {
                                        let job: Job = serde_json::from_slice(&message.data)?;
                                        info!("Received a new job event: {}", hash!(&job));
                                    }
                                    // Received a picked-job message from the network
                                    if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::PickedJob).into() {
                                        let job: Job = serde_json::from_slice(&message.data)?;
                                        info!("Received picked job event: {}", hash!(&job));
                                    }
                                    // Received a finished-job message from the network
                                    if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::FinishedJob).into() {
                                        let job_witness: JobWitness = serde_json::from_slice(&message.data)?;
                                        info!("Received finished job event: {}", &job_witness.job_hash);
                                        job_witness_tx.send(job_witness)?;
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
                        _ = shutdown_signal() => {
                            break
                        }
                        else => break
                    }
                }
                Ok(())
            })),
        }
    }
}

impl Drop for Delegator {
    fn drop(&mut self) {
        let handle = self.handle.take();
        tokio::spawn(async move {
            if let Some(handle) = handle {
                handle.await.unwrap().unwrap();
            }
        });
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DelegatorError {
    #[error("broadcast_send_error")]
    BroadcastSendError(#[from] tokio::sync::broadcast::error::SendError<JobWitness>),

    #[error("io")]
    Io(#[from] std::io::Error),

    #[error("serde")]
    Serde(#[from] serde_json::Error),
}

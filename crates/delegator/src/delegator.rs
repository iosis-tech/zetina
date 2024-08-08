use futures::executor::block_on;
use futures::Stream;
use libp2p::gossipsub;
use starknet::signers::SigningKey;
use std::pin::Pin;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tokio_stream::StreamExt;
use tracing::{error, info};
use zetina_common::graceful_shutdown::shutdown_signal;
use zetina_common::job::{Job, JobData, JobDelegation};
use zetina_common::job_witness::JobWitness;
use zetina_peer::swarm::{
    DelegationMessage, GossipsubMessage, MarketMessage, PeerBehaviourEvent, Topic,
};

use crate::job_bid_queue::JobBidQueue;

pub struct Delegator {
    handle: Option<JoinHandle<Result<(), Error>>>,
}

impl Delegator {
    pub fn new(
        mut swarm_events: Pin<Box<dyn Stream<Item = PeerBehaviourEvent> + Send>>,
        gossipsub_tx: Sender<GossipsubMessage>,
        mut delegate_rx: mpsc::Receiver<JobData>,
        finished_tx: broadcast::Sender<JobWitness>,
        signing_key: SigningKey,
    ) -> Self {
        Self {
            handle: Some(tokio::spawn(async move {
                let mut job_bid_queue = JobBidQueue::new();
                loop {
                    tokio::select! {
                        Some(job_data) = delegate_rx.recv() => {
                            let job = Job::try_from_job_data(job_data, &signing_key);
                            gossipsub_tx.send(GossipsubMessage {
                                topic: Topic::Market.into(),
                                data: serde_json::to_vec(&MarketMessage::Job(job.to_owned()))?
                            }).await?;
                            job_bid_queue.insert_job(job);
                        },
                        Some(event) = swarm_events.next() => {
                            match event {
                                PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, propagation_source, .. }) => {
                                    if message.topic == Topic::Market.into() {
                                        match serde_json::from_slice::<MarketMessage>(&message.data)? {
                                            MarketMessage::JobBid(job_bid) => {
                                                job_bid_queue.insert_bid(job_bid.to_owned(), propagation_source);
                                                if let Some((job, identity, price)) = job_bid_queue.get_best(job_bid.job_hash) {
                                                    gossipsub_tx.send(GossipsubMessage {
                                                        topic: Topic::Delegation.into(),
                                                        data: serde_json::to_vec(&DelegationMessage::Delegate(JobDelegation{
                                                            identity,
                                                            job,
                                                            price
                                                        }))?
                                                    }).await?;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    if message.topic == Topic::Delegation.into() {
                                        match serde_json::from_slice::<DelegationMessage>(&message.data)? {
                                            DelegationMessage::Finished(job_witness) => {
                                                info!("Received finished job: {}", job_witness.job_hash);
                                                finished_tx.send(job_witness)?;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ = shutdown_signal() => {
                            break
                        }
                        else => break
                    };
                }
                Ok(())
            })),
        }
    }
}

impl Drop for Delegator {
    fn drop(&mut self) {
        let handle = self.handle.take();
        block_on(async move { handle.unwrap().await.unwrap().unwrap() });
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("mpsc_send_error GossipsubMessage")]
    MpscSendErrorGossipsubMessage(#[from] mpsc::error::SendError<GossipsubMessage>),

    #[error("mpsc_send_error JobWitness")]
    BreadcastSendErrorJobWitness(#[from] broadcast::error::SendError<JobWitness>),

    #[error("io")]
    Io(#[from] std::io::Error),

    #[error("serde")]
    Serde(#[from] serde_json::Error),
}

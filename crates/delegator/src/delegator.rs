use crate::bid_queue::{BidControllerError, BidQueue};
use futures::stream::FuturesUnordered;
use futures::Stream;
use libp2p::{gossipsub, PeerId};
use starknet::signers::SigningKey;
use std::collections::{BTreeMap, HashMap};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::pin::Pin;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tokio_stream::StreamExt;
use tracing::{error, info};
use zetina_common::graceful_shutdown::shutdown_signal;
use zetina_common::hash;
use zetina_common::job::{Job, JobData, JobDelegation};
use zetina_common::process::Process;
use zetina_peer::swarm::{
    DelegationMessage, GossipsubMessage, MarketMessage, PeerBehaviourEvent, Topic,
};

pub struct Delegator {
    handle: Option<JoinHandle<Result<(), Error>>>,
}

impl Delegator {
    pub fn new(
        mut swarm_events: Pin<Box<dyn Stream<Item = PeerBehaviourEvent> + Send>>,
        gossipsub_tx: Sender<GossipsubMessage>,
        mut delegate_rx: mpsc::Receiver<JobData>,
        events_tx: broadcast::Sender<(u64, DelegatorEvent)>,
        signing_key: SigningKey,
    ) -> Self {
        Self {
            handle: Some(tokio::spawn(async move {
                let mut job_bid_scheduler = FuturesUnordered::<
                    Process<Result<(Job, BTreeMap<u64, Vec<PeerId>>), BidControllerError>>,
                >::new();
                let mut job_hash_store = HashMap::<u64, mpsc::Sender<(u64, PeerId)>>::new();
                loop {
                    tokio::select! {
                        Some(job_data) = delegate_rx.recv() => {
                            let job = Job::try_from_job_data(job_data, &signing_key);
                            gossipsub_tx.send(GossipsubMessage {
                                topic: Topic::Market.into(),
                                data: serde_json::to_vec(&MarketMessage::Job(job.to_owned()))?
                            }).await?;
                            info!("Propagated job: {} for bidding", hash!(job));
                            let (process, bid_tx) = BidQueue::run(job.to_owned());
                            job_bid_scheduler.push(process);
                            job_hash_store.insert(hash!(job), bid_tx);
                        },
                        Some(event) = swarm_events.next() => {
                            match event {
                                PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, propagation_source, .. }) => {
                                    if message.topic == Topic::Market.into() {
                                        match serde_json::from_slice::<MarketMessage>(&message.data)? {
                                            MarketMessage::JobBid(job_bid) => {
                                                if let Some(bid_tx) =  job_hash_store.get_mut(&job_bid.job_hash) {
                                                    info!("Received job bid: {} price: {} from: {}", job_bid.job_hash, job_bid.price, propagation_source);
                                                    bid_tx.send((job_bid.price, propagation_source)).await?;
                                                    events_tx.send((job_bid.job_hash, DelegatorEvent::BidReceived(propagation_source)))?;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    if message.topic == Topic::Delegation.into() {
                                        match serde_json::from_slice::<DelegationMessage>(&message.data)? {
                                            DelegationMessage::Finished(job_witness) => {
                                                info!("Received finished job: {}", job_witness.job_hash);
                                                events_tx.send((job_witness.job_hash, DelegatorEvent::Finished(job_witness.proof)))?;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        Some(Ok((job, bids))) = job_bid_scheduler.next() => {
                            job_hash_store.remove(&hash!(job));
                            let bid = bids.first_key_value().unwrap();
                            let price = *bid.0;
                            let identity = *bid.1.first().unwrap();
                            info!("Job {} delegated to best bidder: {}", hash!(job), identity);
                            gossipsub_tx.send(GossipsubMessage {
                                topic: Topic::Delegation.into(),
                                data: serde_json::to_vec(&DelegationMessage::Delegate(JobDelegation{identity, job: job.to_owned(), price}))?
                            }).await?;
                            events_tx.send((hash!(job), DelegatorEvent::Delegated(identity)))?;
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
        tokio::spawn(async move {
            if let Some(handle) = handle {
                handle.await.unwrap().unwrap();
            }
        });
    }
}

#[derive(Debug, Clone)]
pub enum DelegatorEvent {
    BidReceived(PeerId),
    Delegated(PeerId),
    Finished(Vec<u8>),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("mpsc_send_error GossipsubMessage")]
    MpscSendErrorGossipsubMessage(#[from] mpsc::error::SendError<GossipsubMessage>),

    #[error("mpsc_send_error DelegatorEvent")]
    BreadcastSendErrorDelegatorEvent(#[from] broadcast::error::SendError<(u64, DelegatorEvent)>),

    #[error("mpsc_send_error JobBid")]
    MpscSendErrorJobBid(#[from] mpsc::error::SendError<(u64, PeerId)>),

    #[error("io")]
    Io(#[from] std::io::Error),

    #[error("serde")]
    Serde(#[from] serde_json::Error),
}

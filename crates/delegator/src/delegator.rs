use crate::bid_queue::{BidControllerError, BidQueue};
use futures::stream::FuturesUnordered;
use futures::Stream;
use libp2p::{gossipsub, kad, PeerId};
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
use zetina_common::job::{Job, JobBid, JobData};
use zetina_common::job_witness::JobWitness;
use zetina_common::process::Process;
use zetina_peer::swarm::{
    DelegationMessage, GossipsubMessage, KademliaMessage, MarketMessage, PeerBehaviourEvent, Topic,
};

pub struct Delegator {
    handle: Option<JoinHandle<Result<(), Error>>>,
}

impl Delegator {
    pub fn new(
        mut swarm_events: Pin<Box<dyn Stream<Item = PeerBehaviourEvent> + Send>>,
        gossipsub_tx: Sender<GossipsubMessage>,
        kademlia_tx: Sender<KademliaMessage>,
        mut delegate_rx: mpsc::Receiver<JobData>,
        events_tx: broadcast::Sender<(kad::RecordKey, DelegatorEvent)>,
        signing_key: SigningKey,
    ) -> Self {
        Self {
            handle: Some(tokio::spawn(async move {
                let mut job_bid_scheduler = FuturesUnordered::<
                    Process<
                        Result<(kad::RecordKey, BTreeMap<u64, Vec<PeerId>>), BidControllerError>,
                    >,
                >::new();
                let mut job_hash_store =
                    HashMap::<kad::RecordKey, mpsc::Sender<(u64, PeerId)>>::new();
                let mut proof_hash_store = HashMap::<kad::RecordKey, kad::RecordKey>::new();

                loop {
                    tokio::select! {
                        Some(job_data) = delegate_rx.recv() => {
                            let job = Job::try_from_job_data(job_data, &signing_key);
                            let job_key = kad::RecordKey::new(&hash!(job).to_be_bytes());
                            kademlia_tx.send(KademliaMessage::PUT(
                                (job_key, serde_json::to_vec(&job)?)
                            )).await?;
                        },
                        Some(event) = swarm_events.next() => {
                            match event {
                                PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. }) => {
                                    if message.topic == Topic::Market.into() {
                                        match serde_json::from_slice::<MarketMessage>(&message.data)? {
                                            MarketMessage::JobBid(job_bid) => {
                                                if let Some(bid_tx) =  job_hash_store.get_mut(&job_bid.job_key) {
                                                    info!("Received job bid: {} price: {} from: {}", hex::encode(&job_bid.job_key), job_bid.price, job_bid.identity);
                                                    bid_tx.send((job_bid.price, job_bid.identity)).await?;
                                                    events_tx.send((job_bid.job_key, DelegatorEvent::BidReceived(job_bid.identity)))?;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    if message.topic == Topic::Delegation.into() {
                                        match serde_json::from_slice::<DelegationMessage>(&message.data)? {
                                            DelegationMessage::Finished(proof_key, job_key) => {
                                                if job_hash_store.remove(&job_key).is_some() {
                                                    info!("Received finished job: {} proof key: {}", hex::encode(&job_key), hex::encode(&proof_key));
                                                    proof_hash_store.insert(proof_key.to_owned(), job_key);
                                                    kademlia_tx.send(KademliaMessage::GET(proof_key)).await?;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                                PeerBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed { result, ..}) => {
                                    match result {
                                        kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                                            gossipsub_tx.send(GossipsubMessage {
                                                topic: Topic::Market.into(),
                                                data: serde_json::to_vec(&MarketMessage::JobBidPropagation(key.to_owned()))?
                                            }).await?;
                                            info!("Propagated job: {} for bidding", hex::encode(&key));
                                            let (process, bid_tx) = BidQueue::run(key.to_owned());
                                            job_bid_scheduler.push(process);
                                            job_hash_store.insert(key, bid_tx);
                                        },
                                        kad::QueryResult::GetRecord(Ok(
                                            kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                                                record: kad::Record { key, value, .. },
                                                ..
                                            })
                                        )) => {
                                            if let Some ((proof_key, job_key)) = proof_hash_store.remove_entry(&key) {
                                                info!("job {} proof with key: {} returned in DHT", hex::encode(&job_key), hex::encode(&proof_key));
                                                let job_witness: JobWitness = serde_json::from_slice(&value)?;
                                                events_tx.send((job_key, DelegatorEvent::Finished(job_witness.proof)))?;
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                        Some(Ok((job_key, bids))) = job_bid_scheduler.next() => {
                            let bid = bids.first_key_value().unwrap();
                            let price = *bid.0;
                            let identity = *bid.1.first().unwrap();
                            info!("Job {} delegated to best bidder: {}", hex::encode(&job_key), identity);
                            gossipsub_tx.send(GossipsubMessage {
                                topic: Topic::Delegation.into(),
                                data: serde_json::to_vec(&DelegationMessage::Delegate(JobBid{identity, job_key: job_key.to_owned(), price}))?
                            }).await?;
                            events_tx.send((job_key, DelegatorEvent::Delegated(identity)))?;
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

    #[error("mpsc_send_error KademliaMessage")]
    MpscSendErrorKademliaMessage(#[from] mpsc::error::SendError<KademliaMessage>),

    #[error("mpsc_send_error DelegatorEvent")]
    BreadcastSendErrorDelegatorEvent(
        #[from] broadcast::error::SendError<(kad::RecordKey, DelegatorEvent)>,
    ),

    #[error("mpsc_send_error JobBid")]
    MpscSendErrorJobBid(#[from] mpsc::error::SendError<(u64, PeerId)>),

    #[error("io")]
    Io(#[from] std::io::Error),

    #[error("serde")]
    Serde(#[from] serde_json::Error),
}

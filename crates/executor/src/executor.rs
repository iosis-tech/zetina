use futures::{stream::FuturesUnordered, Stream};
use libp2p::{gossipsub, kad, PeerId};
use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::pin::Pin;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tokio_stream::StreamExt;
use tracing::{error, info};
use zetina_common::hash;
use zetina_common::job::Job;
use zetina_common::{
    graceful_shutdown::shutdown_signal, job::JobBid, job_trace::JobTrace, job_witness::JobWitness,
    process::Process,
};
use zetina_peer::swarm::{
    DelegationMessage, GossipsubMessage, KademliaMessage, MarketMessage, PeerBehaviourEvent, Topic,
};
use zetina_prover::{
    errors::ProverControllerError, stone_prover::StoneProver, traits::ProverController,
};
use zetina_runner::{
    cairo_runner::CairoRunner, errors::RunnerControllerError, traits::RunnerController,
};

pub struct Executor {
    handle: Option<JoinHandle<Result<(), Error>>>,
}

impl Executor {
    pub fn new(
        identity: PeerId,
        mut swarm_events: Pin<Box<dyn Stream<Item = PeerBehaviourEvent> + Send>>,
        gossipsub_tx: Sender<GossipsubMessage>,
        kademlia_tx: Sender<KademliaMessage>,
        runner: CairoRunner,
        prover: StoneProver,
    ) -> Self {
        Self {
            handle: Some(tokio::spawn(async move {
                let mut runner_scheduler =
                    FuturesUnordered::<Process<'_, Result<JobTrace, RunnerControllerError>>>::new();
                let mut prover_scheduler = FuturesUnordered::<
                    Process<'_, Result<JobWitness, ProverControllerError>>,
                >::new();

                let mut job_hash_store = HashSet::<kad::RecordKey>::new();
                let mut proof_hash_store = HashMap::<kad::RecordKey, kad::RecordKey>::new();

                loop {
                    tokio::select! {
                        Some(event) = swarm_events.next() => {
                            match event {
                                PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. }) => {
                                    if message.topic == Topic::Market.into() {
                                        match serde_json::from_slice::<MarketMessage>(&message.data)? {
                                            MarketMessage::JobBidPropagation(job_key) => {
                                                gossipsub_tx
                                                    .send(GossipsubMessage {
                                                        topic: Topic::Market.into(),
                                                        data: serde_json::to_vec(&MarketMessage::JobBid(JobBid {
                                                            identity,
                                                            job_key,
                                                            price: (runner_scheduler.len() * prover_scheduler.len()) as u64,
                                                        }))?
                                                    })
                                                    .await?
                                            }
                                            _ => {}
                                        }
                                    }
                                    if message.topic == Topic::Delegation.into() {
                                        match serde_json::from_slice::<DelegationMessage>(&message.data)? {
                                            DelegationMessage::Delegate(job_delegation) => {
                                                if job_delegation.identity == identity {
                                                    info!("received delegation of job: {}", hex::encode(&job_delegation.job_key));
                                                    job_hash_store.insert(job_delegation.job_key.to_owned());
                                                    kademlia_tx.send(KademliaMessage::GET(job_delegation.job_key)).await?;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                PeerBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed { result, ..}) => {
                                    match result {
                                        kad::QueryResult::GetRecord(Ok(
                                            kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                                                record: kad::Record { key, value, .. },
                                                ..
                                            })
                                        )) => {
                                            if job_hash_store.remove(&key) {
                                                let job: Job = serde_json::from_slice(&value)?;
                                                info!("received delegation of job: {}", hex::encode(&key));
                                                runner_scheduler.push(runner.run(job)?);
                                            }
                                        },
                                        kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                                            if let Some ((proof_key, job_key)) = proof_hash_store.remove_entry(&key) {
                                                info!("job {} proof with key: {} stored in DHT", hex::encode(&job_key), hex::encode(&proof_key));
                                                gossipsub_tx.send(GossipsubMessage {
                                                    topic: Topic::Delegation.into(),
                                                    data: serde_json::to_vec(&DelegationMessage::Finished(proof_key, job_key))?
                                                }).await?;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                        Some(Ok(job_trace)) = runner_scheduler.next() => {
                            info!("Scheduled proving of job_trace: {}", hex::encode(&job_trace.job_key));
                            prover_scheduler.push(prover.run(job_trace)?);
                        },
                        Some(Ok(job_witness)) = prover_scheduler.next() => {
                            let proof_key = kad::RecordKey::new(&hash!(job_witness).to_be_bytes());
                            info!("Finished proving job: {} proof key: {}", hex::encode(&job_witness.job_key), hex::encode(&proof_key));
                            proof_hash_store.insert(proof_key.to_owned(), job_witness.job_key.to_owned());
                            kademlia_tx.send(KademliaMessage::PUT(
                                (proof_key, serde_json::to_vec(&job_witness)?)
                            )).await?;
                        },
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

impl Drop for Executor {
    fn drop(&mut self) {
        let handle = self.handle.take();
        tokio::spawn(async move {
            if let Some(handle) = handle {
                handle.await.unwrap().unwrap();
            }
        });
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("prover_controller_error")]
    ProverControllerError(#[from] ProverControllerError),

    #[error("runner_controller_error")]
    RunnerControllerError(#[from] RunnerControllerError),

    #[error("mpsc_send_error GossipsubMessage")]
    MpscSendErrorGossipsubMessage(#[from] mpsc::error::SendError<GossipsubMessage>),

    #[error("mpsc_send_error KademliaMessage")]
    MpscSendErrorKademliaMessage(#[from] mpsc::error::SendError<KademliaMessage>),

    #[error("io")]
    Io(#[from] std::io::Error),

    #[error("serde")]
    Serde(#[from] serde_json::Error),
}

use futures::{stream::FuturesUnordered, Stream};
use libp2p::{gossipsub, PeerId};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::pin::Pin;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tokio_stream::StreamExt;
use tracing::{error, info};
use zetina_common::{
    graceful_shutdown::shutdown_signal, hash, job::JobBid, job_trace::JobTrace,
    job_witness::JobWitness, process::Process,
};
use zetina_peer::swarm::{
    DelegationMessage, GossipsubMessage, MarketMessage, PeerBehaviourEvent, Topic,
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

                loop {
                    tokio::select! {
                        Some(event) = swarm_events.next() => {
                            match event {
                                PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. }) => {
                                    if message.topic == Topic::Market.into() {
                                        match serde_json::from_slice::<MarketMessage>(&message.data)? {
                                            MarketMessage::Job(job) => {
                                                gossipsub_tx
                                                    .send(GossipsubMessage {
                                                        topic: Topic::Market.into(),
                                                        data: serde_json::to_vec(&MarketMessage::JobBid(JobBid {
                                                            job_hash: hash!(job),
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
                                                    info!("Scheduled running of job: {}", hash!(job_delegation.job));
                                                    runner_scheduler.push(runner.run(job_delegation.job)?);
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        Some(Ok(job_trace)) = runner_scheduler.next() => {
                            info!("Scheduled proving of job_trace: {}", &job_trace.job_hash);
                            prover_scheduler.push(prover.run(job_trace)?);
                        },
                        Some(Ok(job_witness)) = prover_scheduler.next() => {
                            info!("Finished proving: {}", &job_witness.job_hash);
                            gossipsub_tx.send(GossipsubMessage {
                                topic: Topic::Delegation.into(),
                                data: serde_json::to_vec(&DelegationMessage::Finished(job_witness))?
                            }).await?;
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

    #[error("mpsc_send_error")]
    MpscSendError(#[from] mpsc::error::SendError<GossipsubMessage>),

    #[error("io")]
    Io(#[from] std::io::Error),

    #[error("serde")]
    Serde(#[from] serde_json::Error),
}

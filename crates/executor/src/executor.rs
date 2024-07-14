use futures::stream::FuturesUnordered;
use futures::StreamExt;
use libp2p::gossipsub::Event;
use std::hash::{DefaultHasher, Hash, Hasher};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::info;
use zetina_common::{
    graceful_shutdown::shutdown_signal,
    hash,
    job::Job,
    job_record::JobRecord,
    job_trace::JobTrace,
    job_witness::JobWitness,
    network::Network,
    process::Process,
    topic::{gossipsub_ident_topic, Topic},
};
use zetina_prover::errors::ProverControllerError;
use zetina_prover::stone_prover::StoneProver;
use zetina_prover::traits::ProverController;
use zetina_runner::cairo_runner::CairoRunner;
use zetina_runner::errors::RunnerControllerError;
use zetina_runner::traits::RunnerController;

const MAX_PARALLEL_JOBS: usize = 1;

pub struct Executor {
    handle: Option<JoinHandle<Result<(), ExecutorError>>>,
}

impl Executor {
    pub fn new(
        mut events_rx: mpsc::Receiver<Event>,
        finished_job_topic_tx: mpsc::Sender<Vec<u8>>,
        picked_job_topic_tx: mpsc::Sender<Vec<u8>>,
        runner: CairoRunner,
        prover: StoneProver,
    ) -> Self {
        Self {
            handle: Some(tokio::spawn(async move {
                let mut job_record = JobRecord::<Job>::new();
                let mut runner_scheduler =
                    FuturesUnordered::<Process<'_, Result<JobTrace, RunnerControllerError>>>::new();
                let mut prover_scheduler = FuturesUnordered::<
                    Process<'_, Result<JobWitness, ProverControllerError>>,
                >::new();

                loop {
                    tokio::select! {
                        Some(event) = events_rx.recv() => {
                            match event {
                                Event::Message { message, .. } => {
                                    // Received a new-job message from the network
                                    if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::NewJob).into() {
                                        let job: Job = serde_json::from_slice(&message.data)?;
                                        info!("Received a new job event: {}", hash!(&job));
                                        job_record.register_job(job);

                                    }
                                    // Received a picked-job message from the network
                                    if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::PickedJob).into() {
                                        let job: Job = serde_json::from_slice(&message.data)?;
                                        info!("Received picked job event: {}", hash!(&job));
                                        job_record.remove_job(&job);
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
                        Some(Ok(job_trace)) = runner_scheduler.next() => {
                            info!("Scheduled proving of job_trace: {}", &job_trace.job_hash);
                            prover_scheduler.push(prover.run(job_trace)?);
                        },
                        Some(Ok(job_witness)) = prover_scheduler.next() => {
                            info!("Calculated job_witness: {}", &job_witness.job_hash);
                            let serialized_job_witness = serde_json::to_string(&job_witness)?;
                            finished_job_topic_tx.send(serialized_job_witness.into()).await?;
                        },
                        _ = shutdown_signal() => {
                            break
                        }
                        else => break
                    };

                    if runner_scheduler.len() + prover_scheduler.len() < MAX_PARALLEL_JOBS
                        && !job_record.is_empty()
                    {
                        if let Some(job) = job_record.take_job().await {
                            let serialized_job = serde_json::to_string(&job)?;
                            picked_job_topic_tx.send(serialized_job.into()).await?;
                            info!("Sent picked job event: {}", hash!(&job));

                            info!("Scheduled run of job: {}", hash!(&job));
                            runner_scheduler.push(runner.run(job)?);
                        }
                    }
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

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("prover_controller_error")]
    ProverControllerError(#[from] ProverControllerError),

    #[error("runner_controller_error")]
    RunnerControllerError(#[from] RunnerControllerError),

    #[error("mpsc_send_error")]
    MpscSendError(#[from] tokio::sync::mpsc::error::SendError<Vec<u8>>),

    #[error("io")]
    Io(#[from] std::io::Error),

    #[error("serde")]
    Serde(#[from] serde_json::Error),
}

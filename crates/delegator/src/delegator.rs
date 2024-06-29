pub mod event_loop;

use event_loop::delegator_loop;
use futures::executor::block_on;
use futures::FutureExt;
use libp2p::gossipsub::Event;
use starknet::signers::SigningKey;
use std::hash::{DefaultHasher, Hash, Hasher};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::info;
use zetina_common::{
    hash,
    job::{Job, JobData},
    job_witness::JobWitness,
};

pub struct Delegator<'identity> {
    signing_key: &'identity SigningKey,
    job_topic_tx: mpsc::Sender<Vec<u8>>,
    cancellation_token: CancellationToken,
    handle: Option<JoinHandle<()>>,
}

impl<'identity> Delegator<'identity> {
    pub fn new(
        signing_key: &'identity SigningKey,
        job_topic_tx: mpsc::Sender<Vec<u8>>,
        job_witness_tx: mpsc::Sender<JobWitness>,
        events_rx: mpsc::Receiver<Event>,
    ) -> Self {
        let cancellation_token = CancellationToken::new();

        Self {
            signing_key,
            job_topic_tx,
            cancellation_token: cancellation_token.to_owned(),
            handle: Some(tokio::spawn(async move {
                delegator_loop(events_rx, job_witness_tx, cancellation_token).boxed().await
            })),
        }
    }

    pub async fn delegate(self, job_data: JobData) -> Result<(), mpsc::error::SendError<Vec<u8>>> {
        let job = Job::try_from_job_data(job_data, self.signing_key);
        let serialized_job = serde_json::to_string(&job).unwrap();
        self.job_topic_tx.send(serialized_job.into()).await?;
        info!("Sent a new job: {}", hash!(&job));
        Ok(())
    }
}

impl<'identity> Drop for Delegator<'identity> {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
        block_on(async move {
            self.handle.take().unwrap().await.unwrap();
        })
    }
}

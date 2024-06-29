pub mod event_loop;

use event_loop::delegator_loop;
use futures::executor::block_on;
use futures::FutureExt;
use libp2p::gossipsub::Event;
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use zetina_common::job_witness::JobWitness;

pub struct Delegator {
    cancellation_token: CancellationToken,
    handle: Option<JoinHandle<()>>,
}

impl Delegator {
    pub fn new(
        job_witness_tx: broadcast::Sender<JobWitness>,
        events_rx: mpsc::Receiver<Event>,
    ) -> Self {
        let cancellation_token = CancellationToken::new();

        Self {
            cancellation_token: cancellation_token.to_owned(),
            handle: Some(tokio::spawn(async move {
                delegator_loop(events_rx, job_witness_tx, cancellation_token).boxed().await
            })),
        }
    }
}

impl Drop for Delegator {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
        block_on(async move {
            self.handle.take().unwrap().await.unwrap();
        })
    }
}

pub mod proto {
    tonic::include_proto!("delegator");
}
use proto::delegator_service_server::DelegatorService;
use proto::events_response::Event;
use proto::{
    DelegateRequest, DelegateResponse, EventType, EventsRequest, EventsResponse, Picked, Proven,
};

use async_stream::stream;
use futures::{Stream, StreamExt};
use starknet::signers::SigningKey;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::pin::Pin;
use tokio::sync::{broadcast, mpsc};
use tonic::Response;
use tracing::info;
use zetina_common::hash;
use zetina_common::{
    job::{Job, JobData},
    job_witness::JobWitness,
};

pub struct DelegatorGRPCServer {
    signing_key: SigningKey,
    job_topic_tx: mpsc::Sender<Vec<u8>>,
    job_picked_rx: broadcast::Receiver<Job>,
    job_witness_rx: broadcast::Receiver<JobWitness>,
}

impl DelegatorGRPCServer {
    pub fn new(
        signing_key: SigningKey,
        job_topic_tx: mpsc::Sender<Vec<u8>>,
        job_picked_rx: broadcast::Receiver<Job>,
        job_witness_rx: broadcast::Receiver<JobWitness>,
    ) -> Self {
        Self { signing_key, job_topic_tx, job_picked_rx, job_witness_rx }
    }
}

#[tonic::async_trait]
impl DelegatorService for DelegatorGRPCServer {
    type EventsStream =
        Pin<Box<dyn Stream<Item = Result<EventsResponse, tonic::Status>> + Send + 'static>>;

    async fn delegate(
        &self,
        request: tonic::Request<DelegateRequest>,
    ) -> Result<tonic::Response<DelegateResponse>, tonic::Status> {
        let cairo_pie = request.into_inner().cairo_pie;
        let job_data = JobData::new(0, cairo_pie);
        let job = Job::try_from_job_data(job_data, &self.signing_key);
        let serialized_job = serde_json::to_string(&job).unwrap();
        self.job_topic_tx.send(serialized_job.into()).await.unwrap();
        info!("Sent a new job: {}", hash!(&job));
        Ok(Response::new(DelegateResponse { job_hash: hash!(&job) }))
    }

    async fn events(
        &self,
        request: tonic::Request<EventsRequest>,
    ) -> Result<tonic::Response<Self::EventsStream>, tonic::Status> {
        let job_hash = request.into_inner().job_hash;
        let mut job_picked_rx = self.job_picked_rx.resubscribe();
        let mut job_witness_rx = self.job_witness_rx.resubscribe();
        let out_stream: Self::EventsStream = stream! {
            loop {
                tokio::select! {
                    Ok(job) = job_picked_rx.recv() => {
                        if hash!(job) == job_hash {
                            info!("Picked job sent via stream: {}", hash!(&job));
                            yield Ok(EventsResponse{
                                event_type: EventType::Picked.into(),
                                event: Some(Event::Picked(Picked{}))
                            })
                        }
                    },
                    Ok(job_witness) = job_witness_rx.recv() => {
                        if job_witness.job_hash == job_hash {
                            info!("Proven job sent via stream: {}", job_witness.job_hash);
                            yield Ok(EventsResponse{
                                event_type: EventType::Proven.into(),
                                event: Some(Event::Proven(Proven{proof: job_witness.proof}))
                            })
                        }
                    }
                    else => break
                }
            }
        }
        .boxed();
        Ok(Response::new(out_stream))
    }
}

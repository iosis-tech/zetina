use async_stream::stream;
use futures::{Stream, StreamExt, TryStreamExt};
use starknet::signers::SigningKey;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::pin::Pin;
use tokio::{
    select,
    sync::{broadcast, mpsc},
};
use tonic::{Request, Response, Status, Streaming};
use tracing::info;
use zetina_common::hash;
use zetina_common::{
    job::{Job, JobData},
    job_witness::JobWitness,
};

pub mod proto {
    tonic::include_proto!("delegator");
}
use crate::tonic::proto::delegator_service_server::DelegatorService;
use proto::{DelegateRequest, DelegateResponse};

pub struct DelegatorGRPCServer {
    signing_key: SigningKey,
    job_topic_tx: mpsc::Sender<Vec<u8>>,
    job_witness_rx: broadcast::Receiver<JobWitness>,
}

impl DelegatorGRPCServer {
    pub fn new(
        signing_key: SigningKey,
        job_topic_tx: mpsc::Sender<Vec<u8>>,
        job_witness_rx: broadcast::Receiver<JobWitness>,
    ) -> Self {
        Self { signing_key, job_topic_tx, job_witness_rx }
    }
}

#[tonic::async_trait]
impl DelegatorService for DelegatorGRPCServer {
    type DelegatorStream = Pin<Box<dyn Stream<Item = Result<DelegateResponse, Status>> + Send>>;
    async fn delegator(
        &self,
        request: Request<Streaming<DelegateRequest>>,
    ) -> Result<Response<Self::DelegatorStream>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let mut in_stream = request.into_inner().into_stream().fuse();
        let job_channel = self.job_topic_tx.clone();
        let mut witness_channel = self.job_witness_rx.resubscribe();
        let signing_key = self.signing_key.clone();

        let out_stream = stream! {
            loop {
                select! {
                    Ok(request) = in_stream.select_next_some() => {
                        let job_data = JobData::new(0, request.cairo_pie);
                        let job = Job::try_from_job_data(job_data, &signing_key);
                        let serialized_job = serde_json::to_string(&job).unwrap();
                        job_channel.send(serialized_job.into()).await.unwrap();
                        info!("Sent a new job: {}", hash!(&job));
                    }
                    Ok(rx) = witness_channel.recv() => {
                        yield Ok(DelegateResponse { proof: rx.proof })
                    }
                    else => {
                        yield Err(Status::cancelled(""))
                    }
                }
            }
        }
        .boxed();

        Ok(Response::new(out_stream))
    }
}

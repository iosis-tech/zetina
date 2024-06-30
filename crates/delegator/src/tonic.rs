pub mod proto {
    tonic::include_proto!("delegator");
}
use proto::delegator_service_server::DelegatorService;
use proto::{DelegateRequest, DelegateResponse};

use async_stream::stream;
use futures::{Stream, StreamExt, TryStreamExt};
use starknet::signers::SigningKey;
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::pin::Pin;
use tokio::sync::{broadcast, mpsc};
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error, info};
use zetina_common::hash;
use zetina_common::{
    job::{Job, JobData},
    job_witness::JobWitness,
};

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
        info!("Got a request from {:?}", request.remote_addr());
        let mut in_stream = request.into_inner().into_stream().fuse();
        let job_channel = self.job_topic_tx.clone();
        let mut witness_channel = self.job_witness_rx.resubscribe();
        let signing_key = self.signing_key.clone();

        let mut queue_set = HashSet::<u64>::new();

        let out_stream = stream! {
            loop {
                tokio::select! {
                    Some(request) = in_stream.next() => {
                        match request {
                            Ok(r) => {
                                let job_data = JobData::new(0, r.cairo_pie);
                                let job = Job::try_from_job_data(job_data, &signing_key);
                                queue_set.insert(hash!(job));
                                let serialized_job = serde_json::to_string(&job).unwrap();
                                job_channel.send(serialized_job.into()).await.unwrap();
                                info!("Sent a new job: {}", hash!(&job));
                            }
                            Err(err) => error!("Error: {}",err)
                        }
                    }
                    Ok(rx) = witness_channel.recv() => {
                        debug!("Received job witness: {}", &rx.job_hash);
                        if let Some(job_hash) = queue_set.take(&rx.job_hash) {
                            info!("Received awaited job witness: {}", &job_hash);
                            yield Ok(DelegateResponse { job_hash, proof: rx.proof })
                        }
                    }
                    else => {
                        error!("Stream cancelled!");
                        yield Err(Status::cancelled(""))
                    }
                }
            }
        }
        .boxed();

        Ok(Response::new(out_stream))
    }
}

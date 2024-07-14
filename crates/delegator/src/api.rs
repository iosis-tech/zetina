use async_stream::stream;
use axum::{
    extract::{Query, State},
    response::{sse::Event, Sse},
    Json,
};
use futures::StreamExt;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use starknet::signers::SigningKey;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::{io, time::Duration};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::Stream;
use tracing::info;
use zetina_common::{
    hash,
    job::{Job, JobData},
    job_witness::JobWitness,
};

#[derive(Debug)]
pub struct ServerState {
    pub signing_key: SigningKey,
    pub job_topic_tx: mpsc::Sender<Vec<u8>>,
    pub job_picked_rx: broadcast::Receiver<Job>,
    pub job_witness_rx: broadcast::Receiver<JobWitness>,
}

impl Clone for ServerState {
    fn clone(&self) -> Self {
        Self {
            signing_key: self.signing_key.to_owned(),
            job_topic_tx: self.job_topic_tx.clone(),
            job_picked_rx: self.job_picked_rx.resubscribe(),
            job_witness_rx: self.job_witness_rx.resubscribe(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DelegateRequest {
    trace: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct DelegateResponse {
    job_hash: String,
}

pub async fn deletage_handler(
    State(state): State<ServerState>,
    Json(input): Json<DelegateRequest>,
) -> Result<Json<DelegateResponse>, StatusCode> {
    let cairo_pie = input.trace;
    let job_data = JobData::new(0, cairo_pie);
    let job = Job::try_from_job_data(job_data, &state.signing_key);
    let serialized_job = serde_json::to_string(&job).unwrap();
    state.job_topic_tx.send(serialized_job.into()).await.unwrap();
    info!("Sent a new job: {}", hash!(&job));
    Ok(Json(DelegateResponse { job_hash: hash!(&job).to_string() }))
}

#[derive(Debug, Deserialize)]
pub struct JobEventsRequest {
    job_hash: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum JobEventsResponse {
    Picked(String),
    Witness(Vec<u8>),
}

pub async fn job_events_handler(
    State(mut state): State<ServerState>,
    Query(input): Query<JobEventsRequest>,
) -> Sse<impl Stream<Item = Result<Event, io::Error>>> {
    let stream = stream! {
        let job_hash = input.job_hash.parse::<u64>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        loop {
            tokio::select! {
                Ok(job) = state.job_picked_rx.recv() => {
                    info!("Received job picked: {}", hash!(job));
                    if hash!(job) == job_hash {
                        yield Event::default()
                            .json_data(JobEventsResponse::Picked(hash!(job).to_string()))
                            .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e.to_string()));
                    }
                },
                Ok(job_witness) = state.job_witness_rx.recv() => {
                    info!("Received job witness: {}", &job_witness.job_hash);
                    if job_witness.job_hash == job_hash {
                        yield Event::default()
                            .json_data(JobEventsResponse::Witness(job_witness.proof))
                            .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e.to_string()));
                    }
                }
                else => break
            }
        }
    }
    .boxed();

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(5))
            .text("keep-alive-text"),
    )
}

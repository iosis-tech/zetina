use async_stream::stream;
use axum::{
    extract::{Query, State},
    response::{sse::Event, Sse},
    Json,
};
use futures::StreamExt;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::{io, time::Duration};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::Stream;
use zetina_common::{hash, job::JobData, job_witness::JobWitness};

#[derive(Debug)]
pub struct ServerState {
    pub delegate_tx: mpsc::Sender<JobData>,
    pub finished_rx: broadcast::Receiver<JobWitness>,
}

impl Clone for ServerState {
    fn clone(&self) -> Self {
        Self {
            delegate_tx: self.delegate_tx.to_owned(),
            finished_rx: self.finished_rx.resubscribe(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DelegateRequest {
    pie: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct DelegateResponse {
    job_hash: String,
}

pub async fn deletage_handler(
    State(state): State<ServerState>,
    Json(input): Json<DelegateRequest>,
) -> Result<Json<DelegateResponse>, StatusCode> {
    let job_data = JobData::new(input.pie);
    let job_data_hash = hash!(&job_data);
    state.delegate_tx.send(job_data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(DelegateResponse { job_hash: job_data_hash.to_string() }))
}

#[derive(Debug, Deserialize)]
pub struct JobEventsRequest {
    job_hash: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum JobEventsResponse {
    Finished(Vec<u8>),
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
                Ok(job_witness) = state.finished_rx.recv() => {
                    if job_witness.job_hash == job_hash {
                        yield Event::default()
                            .json_data(JobEventsResponse::Finished(job_witness.proof))
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

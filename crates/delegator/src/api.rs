use async_stream::stream;
use axum::{
    extract::{Query, State},
    response::{sse::Event, IntoResponse, Sse},
    Json,
};
use futures::StreamExt;
use hyper::StatusCode;
use libp2p::kad;
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::{io, time::Duration};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::Stream;
use zetina_common::{hash, job::JobData};

use crate::delegator::DelegatorEvent;

#[derive(Debug)]
pub struct ServerState {
    pub delegate_tx: mpsc::Sender<JobData>,
    pub events_rx: broadcast::Receiver<(kad::RecordKey, DelegatorEvent)>,
}

impl Clone for ServerState {
    fn clone(&self) -> Self {
        Self { delegate_tx: self.delegate_tx.to_owned(), events_rx: self.events_rx.resubscribe() }
    }
}

pub async fn health_check_handler() -> impl IntoResponse {
    (StatusCode::OK, "Health check: OK")
}

#[derive(Debug, Deserialize)]
pub struct DelegateRequest {
    pie: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct DelegateResponse {
    job_key: String,
}

pub async fn deletage_handler(
    State(state): State<ServerState>,
    Json(input): Json<DelegateRequest>,
) -> Result<Json<DelegateResponse>, StatusCode> {
    let job_data = JobData::new(input.pie);
    let job_data_hash = kad::RecordKey::new(&hash!(job_data).to_be_bytes());
    state.delegate_tx.send(job_data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(DelegateResponse { job_key: hex::encode(job_data_hash) }))
}

#[derive(Debug, Deserialize)]
pub struct JobEventsRequest {
    job_key: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum JobEventsResponse {
    BidReceived(String),
    Delegated(String),
    Finished(Vec<u8>),
}

pub async fn job_events_handler(
    State(mut state): State<ServerState>,
    Query(input): Query<JobEventsRequest>,
) -> Sse<impl Stream<Item = Result<Event, io::Error>>> {
    let stream = stream! {
        let job_key =  kad::RecordKey::new(
            &hex::decode(input.job_key)
            .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e.to_string()))?
        );
        loop {
            tokio::select! {
                Ok((key, event)) = state.events_rx.recv() => {
                    if key == job_key {
                        yield Event::default()
                            .json_data(
                                match event {
                                    DelegatorEvent::BidReceived(peer_id) => { JobEventsResponse::BidReceived(peer_id.to_base58()) },
                                    DelegatorEvent::Delegated(peer_id) => { JobEventsResponse::Delegated(peer_id.to_base58()) },
                                    DelegatorEvent::Finished(data) => { JobEventsResponse::Finished(data) },
                                }
                            )
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

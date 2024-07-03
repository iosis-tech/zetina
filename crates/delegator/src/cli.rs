use clap::{command, Parser};
use libp2p::gossipsub;
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient, Url};
use tokio::sync::{broadcast, mpsc};
use tonic::transport::Server;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use zetina_common::{
    graceful_shutdown::shutdown_signal,
    job_witness::JobWitness,
    network::Network,
    node_account::NodeAccount,
    topic::{gossipsub_ident_topic, Topic},
};

use crate::{
    behavior_handler::BehaviourHandler,
    swarm::SwarmRunner,
    tonic::{proto::delegator_service_server::DelegatorServiceServer, DelegatorGRPCServer},
};

#[derive(Debug, Parser)]
#[command(name = "zetina-submit")]
#[command(version, about, long_about = None)]
pub struct DelegatorCommand {
    pub network: Network,
    pub private_key: String,
    pub account_address: String,
    pub rpc_url: Url,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let delegator_command = DelegatorCommand::parse();
    let DelegatorCommand { network, private_key, account_address, rpc_url } = delegator_command;
    let node_account = NodeAccount::new(
        hex::decode(private_key)?,
        hex::decode(account_address)?,
        network,
        JsonRpcClient::new(HttpTransport::new(rpc_url)),
    );

    // Generate topic
    let new_job_topic = gossipsub_ident_topic(network, Topic::NewJob);
    let picked_job_topic = gossipsub_ident_topic(network, Topic::PickedJob);
    let finished_job_topic = gossipsub_ident_topic(network, Topic::FinishedJob);

    let (swarm_events_tx, swarm_events_rx) = mpsc::channel::<gossipsub::Event>(100);
    let (job_witness_tx, job_witness_rx) = broadcast::channel::<JobWitness>(100);

    let (new_job_topic_tx, new_job_topic_rx) = mpsc::channel::<Vec<u8>>(100);

    SwarmRunner::new(
        node_account.get_keypair(),
        vec![new_job_topic.to_owned(), picked_job_topic, finished_job_topic],
        vec![(new_job_topic.to_owned(), new_job_topic_rx)],
        swarm_events_tx,
    )?;

    BehaviourHandler::new(job_witness_tx, swarm_events_rx);

    let server = DelegatorGRPCServer::new(
        node_account.get_signing_key().to_owned(),
        new_job_topic_tx,
        job_witness_rx,
    );

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<DelegatorServiceServer<DelegatorGRPCServer>>().await;

    Server::builder()
        .add_service(health_service)
        .add_service(DelegatorServiceServer::new(server))
        .serve_with_shutdown("[::]:50051".parse().unwrap(), shutdown_signal())
        .await?;

    Ok(())
}

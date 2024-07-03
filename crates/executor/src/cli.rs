use std::path::PathBuf;

use clap::Parser;
use libp2p::gossipsub;
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient, Url};
use tokio::sync::mpsc;
use tonic::transport::Server;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use zetina_common::{
    graceful_shutdown::shutdown_signal,
    network::Network,
    node_account::NodeAccount,
    topic::{gossipsub_ident_topic, Topic},
};
use zetina_prover::stone_prover::StoneProver;
use zetina_runner::cairo_runner::CairoRunner;

use crate::{
    behavior_handler::BehaviourHandler,
    swarm::SwarmRunner,
    tonic::{proto::executor_service_server::ExecutorServiceServer, ExecutorGRPCServer},
};

#[derive(Debug, Parser)]
#[command(name = "zetina-executor")]
#[command(version, about, long_about = None)]
pub struct ExecutorCommand {
    pub network: Network,
    pub private_key: String,
    pub account_address: String,
    pub rpc_url: Url,
    pub bootloader_program_path: PathBuf,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let executor_command = ExecutorCommand::parse();
    let ExecutorCommand { network, private_key, account_address, rpc_url, bootloader_program_path } =
        executor_command;

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
    let (picked_job_topic_tx, picked_job_topic_rx) = mpsc::channel::<Vec<u8>>(1000);
    let (finished_job_topic_tx, finished_job_topic_rx) = mpsc::channel::<Vec<u8>>(1000);

    SwarmRunner::new(
        node_account.get_keypair(),
        vec![new_job_topic, picked_job_topic.to_owned(), finished_job_topic.to_owned()],
        vec![
            (picked_job_topic.to_owned(), picked_job_topic_rx),
            (finished_job_topic, finished_job_topic_rx),
        ],
        swarm_events_tx,
    )?;

    let verifying_key = node_account.get_verifying_key();
    let runner = CairoRunner::new(bootloader_program_path, verifying_key);
    let prover = StoneProver::new();

    BehaviourHandler::new(
        swarm_events_rx,
        finished_job_topic_tx,
        picked_job_topic_tx,
        runner,
        prover,
    );

    let server = ExecutorGRPCServer::default();

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<ExecutorServiceServer<ExecutorGRPCServer>>().await;

    Server::builder()
        .add_service(health_service)
        .add_service(ExecutorServiceServer::new(server))
        .serve_with_shutdown("[::]:50052".parse().unwrap(), shutdown_signal())
        .await?;

    Ok(())
}

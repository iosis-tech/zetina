use clap::Parser;
use libp2p::gossipsub;
use starknet::{
    core::types::FieldElement,
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Url},
};
use tokio::sync::mpsc;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;
use zetina_common::{
    graceful_shutdown::shutdown_signal,
    network::Network,
    node_account::NodeAccount,
    topic::{gossipsub_ident_topic, Topic},
    value_parser::{parse_bytes, parse_field_element},
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
    #[arg(value_parser = parse_bytes)]
    pub private_key: Vec<u8>,
    #[arg(value_parser = parse_field_element)]
    pub account_address: FieldElement,
    pub rpc_url: Url,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();
    let executor_command = ExecutorCommand::parse();
    let ExecutorCommand { network, private_key, account_address, rpc_url } = executor_command;
    let ws_root = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"),
    )
    .join("../../");
    let bootloader_program_path = ws_root.join("target/bootloader.json");

    // // TODO: common setup in node initiate binary
    // let network = Network::Sepolia;
    // let private_key =
    //     hex::decode("07c7a41c77c7a3b19e7c77485854fc88b09ed7041361595920009f81236d55d2")?;
    // let account_address =
    //     hex::decode("cdd51fbc4e008f4ef807eaf26f5043521ef5931bbb1e04032a25bd845d286b")?;
    // let url = "https://starknet-sepolia.public.blastapi.io";

    let node_account = NodeAccount::new(
        private_key,
        account_address,
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

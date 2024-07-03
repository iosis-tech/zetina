use clap::{command, Parser};
use libp2p::gossipsub;
use starknet::{
    core::types::FieldElement,
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Url},
};
use tokio::sync::{broadcast, mpsc};
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;
use zetina_common::{
    graceful_shutdown::shutdown_signal,
    job_witness::JobWitness,
    network::Network,
    node_account::NodeAccount,
    topic::{gossipsub_ident_topic, Topic},
    value_parser::{parse_bytes, parse_field_element},
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
    #[arg(value_parser = parse_bytes)]
    pub private_key: Vec<u8>,
    #[arg(value_parser = parse_field_element)]
    pub account_address: FieldElement,
    pub rpc_url: Url,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();
    let delegator_command = DelegatorCommand::parse();
    let DelegatorCommand { network, private_key, account_address, rpc_url } = delegator_command;

    // // TODO: common setup in node initiate binary
    // let network = Network::Sepolia;
    // let private_key =
    //     hex::decode("018ef9563461ec2d88236d59039babf44c97d8bf6200d01d81170f1f60a78f32")?;
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

pub mod delegator;
pub mod swarm;
pub mod tonic;

use delegator::Delegator;
use libp2p::gossipsub;
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient, Url};
use swarm::SwarmRunner;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;
use zetina_common::{
    job_witness::JobWitness,
    network::Network,
    node_account::NodeAccount,
    topic::{gossipsub_ident_topic, Topic},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    // TODO: common setup in node initiate binary
    let network = Network::Sepolia;
    let private_key =
        hex::decode("018ef9563461ec2d88236d59039babf44c97d8bf6200d01d81170f1f60a78f32")?;
    let account_address =
        hex::decode("cdd51fbc4e008f4ef807eaf26f5043521ef5931bbb1e04032a25bd845d286b")?;
    let url = "https://starknet-sepolia.public.blastapi.io";

    let node_account = NodeAccount::new(
        private_key,
        account_address,
        network,
        JsonRpcClient::new(HttpTransport::new(Url::parse(url)?)),
    );

    // Generate topic
    let new_job_topic = gossipsub_ident_topic(network, Topic::NewJob);
    let picked_job_topic = gossipsub_ident_topic(network, Topic::PickedJob);
    let finished_job_topic = gossipsub_ident_topic(network, Topic::FinishedJob);

    let (swarm_events_tx, swarm_events_rx) = mpsc::channel::<gossipsub::Event>(100);
    let (job_witness_tx, job_witness_rx) = mpsc::channel::<JobWitness>(100);

    let (new_job_topic_tx, new_job_topic_rx) = mpsc::channel::<Vec<u8>>(100);

    SwarmRunner::new(
        node_account.get_keypair(),
        vec![new_job_topic.to_owned(), picked_job_topic, finished_job_topic],
        vec![(new_job_topic.to_owned(), new_job_topic_rx)],
        swarm_events_tx,
    )?;

    Delegator::new(
        node_account.get_signing_key(),
        new_job_topic_tx,
        job_witness_tx,
        swarm_events_rx,
    );

    Ok(())
}

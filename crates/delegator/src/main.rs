#![deny(unused_crate_dependencies)]

use futures::{stream::FuturesUnordered, StreamExt};
use libp2p::gossipsub::Event;
use sharp_p2p_common::{
    graceful_shutdown::shutdown_signal,
    hash,
    job::Job,
    network::Network,
    node_account::NodeAccount,
    process::Process,
    topic::{gossipsub_ident_topic, Topic},
};
use sharp_p2p_compiler::{
    cairo_compiler::{tests::models::fixture, CairoCompiler},
    errors::CompilerControllerError,
    traits::CompilerController,
};
use sharp_p2p_peer::{registry::RegistryHandler, swarm::SwarmRunner};
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient, Url};
use std::hash::{DefaultHasher, Hash, Hasher};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    sync::mpsc,
};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

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

    let mut registry_handler =
        RegistryHandler::new(JsonRpcClient::new(HttpTransport::new(Url::parse(url)?)));
    let registry_address = registry_handler.get_registry_address();
    let node_account = NodeAccount::new(
        private_key,
        account_address,
        network,
        JsonRpcClient::new(HttpTransport::new(Url::parse(url)?)),
    );

    // Generate topic
    let new_job_topic = gossipsub_ident_topic(network, Topic::NewJob);
    let picked_job_topic = gossipsub_ident_topic(network, Topic::PickedJob);

    let mut swarm_runner = SwarmRunner::new(
        node_account.get_keypair(),
        &[new_job_topic.to_owned(), picked_job_topic],
    )?;

    let (send_topic_tx, send_topic_rx) = mpsc::channel::<Vec<u8>>(1000);
    let mut message_stream = swarm_runner.run(new_job_topic, send_topic_rx);
    let mut event_stream = registry_handler.subscribe_events(vec!["0x0".to_string()]);

    let compiler = CairoCompiler::new(node_account.get_signing_key(), registry_address);

    let mut compiler_scheduler =
        FuturesUnordered::<Process<'_, Result<Job, CompilerControllerError>>>::new();

    // Read cairo program path from stdin
    let mut stdin = BufReader::new(stdin()).lines();

    loop {
        tokio::select! {
            Ok(Some(_)) = stdin.next_line() => {
                // TODO: handle fixture better way
                let fixture = fixture();
                compiler_scheduler.push(compiler.run(fixture.program_path.clone(), fixture.program_input_path)?);
                info!("Scheduled compiling program at path: {:?}", fixture.program_path);

            },
            Some(event) = message_stream.next() => {
                match event {
                    Event::Message { message, .. } => {
                        // Received a new-job message from the network
                        if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::NewJob).into() {
                            let job: Job = serde_json::from_slice(&message.data).unwrap();
                            info!("Received a new job event: {}", hash!(&job));

                        }
                        // Received a picked-job message from the network
                        if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::PickedJob).into() {
                            let job: Job = serde_json::from_slice(&message.data).unwrap();
                            info!("Received picked job event: {}", hash!(&job));
                        }
                    },
                    Event::Subscribed { peer_id, topic } => {
                        info!("{} subscribed to the topic {}", peer_id.to_string(), topic.to_string());
                    },
                    Event::Unsubscribed { peer_id, topic }=> {
                        info!("{} unsubscribed to the topic {}", peer_id.to_string(), topic.to_string());
                    },
                    _ => {}
                }
            },
            Some(Ok(event_vec)) = event_stream.next() => {
                debug!("{:?}", event_vec);
            },
            Some(Ok(job)) = compiler_scheduler.next() => {
                let serialized_job = serde_json::to_string(&job).unwrap();
                send_topic_tx.send(serialized_job.into()).await?;
                info!("Sent a new job: {}", hash!(&job));
            },
            _ = shutdown_signal() => {
                break
            }
            else => break
        }
    }

    Ok(())
}

#![deny(unused_crate_dependencies)]

use futures::{stream::FuturesUnordered, StreamExt};
use libp2p::gossipsub::Event;
use sharp_p2p_common::{
    graceful_shutdown::shutdown_signal,
    hash,
    job::Job,
    job_record::JobRecord,
    job_trace::JobTrace,
    job_witness::JobWitness,
    network::Network,
    node_account::NodeAccount,
    process::Process,
    topic::{gossipsub_ident_topic, Topic},
};
use sharp_p2p_peer::{registry::RegistryHandler, swarm::SwarmRunner};
use sharp_p2p_prover::{
    errors::ProverControllerError, stone_prover::StoneProver, traits::ProverController,
};
use sharp_p2p_runner::{
    cairo_runner::CairoRunner, errors::RunnerControllerError, traits::RunnerController,
};
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient, Url};
use std::hash::{DefaultHasher, Hash, Hasher};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    sync::mpsc,
};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

const MAX_PARALLEL_JOBS: usize = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    let ws_root = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"),
    )
    .join("../../");
    let bootloader_program_path = ws_root.join("target/bootloader.json");

    // TODO: common setup in node initiate binary
    let network = Network::Sepolia;
    let private_key =
        hex::decode("07c7a41c77c7a3b19e7c77485854fc88b09ed7041361595920009f81236d55d2")?;
    let account_address =
        hex::decode("cdd51fbc4e008f4ef807eaf26f5043521ef5931bbb1e04032a25bd845d286b")?;
    let url = "https://starknet-sepolia.public.blastapi.io";

    let mut registry_handler =
        RegistryHandler::new(JsonRpcClient::new(HttpTransport::new(Url::parse(url)?)));
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
        &[new_job_topic, picked_job_topic.to_owned()],
    )?;

    let (send_topic_tx, send_topic_rx) = mpsc::channel::<Vec<u8>>(1000);
    let mut message_stream = swarm_runner.run(picked_job_topic, send_topic_rx);
    let mut event_stream = registry_handler.subscribe_events(vec!["0x0".to_string()]);

    let verifying_key = node_account.get_verifying_key();
    let runner = CairoRunner::new(bootloader_program_path, &verifying_key);
    let prover = StoneProver::new();

    let mut job_record = JobRecord::<Job>::new();
    let mut runner_scheduler =
        FuturesUnordered::<Process<'_, Result<JobTrace, RunnerControllerError>>>::new();
    let mut prover_scheduler =
        FuturesUnordered::<Process<'_, Result<JobWitness, ProverControllerError>>>::new();

    // Read full lines from stdin
    let mut stdin = BufReader::new(stdin()).lines();

    loop {
        tokio::select! {
            Ok(Some(line)) = stdin.next_line() => {
                send_topic_tx.send(line.as_bytes().to_vec()).await?;
            },
            Some(event) = message_stream.next() => {
                match event {
                    Event::Message { message, .. } => {
                        // Received a new-job message from the network
                        if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::NewJob).into() {
                            let job: Job = serde_json::from_slice(&message.data)?;
                            info!("Received a new job event: {}", hash!(&job));
                            job_record.register_job(job);

                        }
                        // Received a picked-job message from the network
                        if message.topic ==  gossipsub_ident_topic(Network::Sepolia, Topic::PickedJob).into() {
                            let job: Job = serde_json::from_slice(&message.data)?;
                            info!("Received picked job event: {}", hash!(&job));
                            job_record.remove_job(&job);
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
            Some(Ok(job_trace)) = runner_scheduler.next() => {
                info!("Scheduled proving of job_trace: {}", hash!(&job_trace));
                prover_scheduler.push(prover.run(job_trace)?);
            },
            Some(Ok(job_witness)) = prover_scheduler.next() => {
                info!("Calculated job_witness: {}", hash!(&job_witness));
            },
            _ = shutdown_signal() => {
                break
            }
            else => break
        };

        if runner_scheduler.len() + prover_scheduler.len() < MAX_PARALLEL_JOBS
            && !job_record.is_empty()
        {
            if let Some(job) = job_record.take_job().await {
                let serialized_job = serde_json::to_string(&job)?;
                send_topic_tx.send(serialized_job.into()).await?;
                info!("Sent picked job event: {}", hash!(&job));

                info!("Scheduled run of job: {}", hash!(&job));
                runner_scheduler.push(runner.run(job)?);
            }
        }
    }

    Ok(())
}

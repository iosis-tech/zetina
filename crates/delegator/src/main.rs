use futures_util::StreamExt;
use sharp_p2p_common::identity::IdentityHandler;
use sharp_p2p_common::network::Network;
use sharp_p2p_common::topic::{gossipsub_ident_topic, Topic};
use sharp_p2p_compiler::cairo_compiler::tests::models::fixture;
use sharp_p2p_compiler::cairo_compiler::CairoCompiler;
use sharp_p2p_compiler::traits::CompilerController;
use sharp_p2p_peer::registry::RegistryHandler;
use sharp_p2p_peer::swarm::SwarmRunner;
use starknet::core::types::FieldElement;

use std::error::Error;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    // Pass the private key to the IdentityHandler
    let private_key = FieldElement::from_hex_be(
        "0139fe4d6f02e666e86a6f58e65060f115cd3c185bd9e98bd829636931458f79",
    )
    .unwrap();
    let identity_handler = IdentityHandler::new(private_key);
    let p2p_local_keypair = identity_handler.get_keypair();

    // Generate topic
    let new_job_topic = gossipsub_ident_topic(Network::Sepolia, Topic::NewJob);
    let picked_job_topic = gossipsub_ident_topic(Network::Sepolia, Topic::PickedJob);

    let mut swarm_runner =
        SwarmRunner::new(&p2p_local_keypair, &[new_job_topic.to_owned(), picked_job_topic])?;
    let mut registry_handler = RegistryHandler::new(
        "https://starknet-sepolia.public.blastapi.io",
        "0xcdd51fbc4e008f4ef807eaf26f5043521ef5931bbb1e04032a25bd845d286b",
    );
    let registry_address = registry_handler.get_registry_address();

    let (send_topic_tx, send_topic_rx) = mpsc::channel::<Vec<u8>>(1000);
    let mut message_stream = swarm_runner.run(new_job_topic, send_topic_rx);
    let mut event_stream = registry_handler.subscribe_events(vec!["0x0".to_string()]);

    // Read cairo program path from stdin
    let mut stdin = BufReader::new(stdin()).lines();
    let compiler = CairoCompiler::new(identity_handler.get_ecdsa_keypair(), registry_address);

    loop {
        tokio::select! {
            Ok(Some(_)) = stdin.next_line() => {
                // TODO: handle fixture better way
                let fixture = fixture();
                let job = compiler.run(fixture.program_path, fixture.program_input_path).unwrap().await.unwrap();
                let serialized_job = serde_json::to_string(&job).unwrap();
                send_topic_tx.send(serialized_job.into()).await?;
            },
            Some(event) = message_stream.next() => {
                info!("{:?}", event);
            },
            Some(Ok(event_vec)) = event_stream.next() => {
                info!("{:?}", event_vec);
            },
            else => break
        }
    }

    Ok(())
}

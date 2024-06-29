use futures::StreamExt;
use libp2p::{
    gossipsub::{self, IdentTopic},
    mdns,
    swarm::SwarmEvent,
    Swarm,
};
use tokio::sync::mpsc::{self, Receiver};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

use crate::swarm::PeerBehaviourEvent;

use super::PeerBehaviour;

pub(crate) async fn swarm_loop(
    mut swarm: Swarm<PeerBehaviour>,
    mut transmit_topics: Vec<(IdentTopic, Receiver<Vec<u8>>)>,
    swarm_events_tx: mpsc::Sender<gossipsub::Event>,
    cancellation_token: CancellationToken,
) {
    loop {
        tokio::select! {
            Some(data) = transmit_topics[0].1.recv() => {
                debug!("Publishing to topic: {:?}", transmit_topics[0].0);
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(transmit_topics[0].0.clone(), data) {
                    error!("Publish error: {e:?}");
                }
            },
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(PeerBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        debug!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(PeerBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        debug!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source,
                    message_id,
                    message,
                })) => {
                    swarm_events_tx.send(gossipsub::Event::Message { propagation_source, message_id, message }).await.unwrap();
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    debug!("Local node is listening on {address}");
                }
                _ => {}
            },
            _ = cancellation_token.cancelled() => {
                break
            }
        }
    }
}

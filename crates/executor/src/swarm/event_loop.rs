use super::PeerBehaviour;
use crate::swarm::PeerBehaviourEvent;
use futures::StreamExt;
use libp2p::{
    gossipsub::{self, IdentTopic},
    mdns,
    swarm::SwarmEvent,
    Swarm,
};
use tokio::sync::mpsc::{self, Receiver};
use tracing::{debug, error};
use zetina_common::graceful_shutdown::shutdown_signal;

pub(crate) async fn swarm_loop(
    mut swarm: Swarm<PeerBehaviour>,
    mut transmit_topics: Vec<(IdentTopic, Receiver<Vec<u8>>)>,
    swarm_events_tx: mpsc::Sender<gossipsub::Event>,
) {
    // TODO make it nicer solution, extensible not manual!
    let mut topic_one = transmit_topics.pop().unwrap();
    let mut topic_two = transmit_topics.pop().unwrap();
    loop {
        tokio::select! {
            Some(data) = topic_one.1.recv() => {
                debug!("Publishing to topic: {:?}", topic_one.0);
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(topic_one.0.clone(), data) {
                    error!("Publish error: {e:?}");
                }
            },
            Some(data) = topic_two.1.recv() => {
                debug!("Publishing to topic: {:?}", topic_two.0);
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(topic_two.0.clone(), data) {
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
            _ = shutdown_signal() => {
                break
            }
        }
    }
}

use crate::registry::{AnnounceMsg, Registry};

use libp2p::{
    gossipsub::{Behaviour as Gossipsub, ConfigBuilder as GossipsubConfigBuilder, Event as GossipsubEvent, IdentTopic, MessageAuthenticity},
    identity::Keypair,
    mdns::{tokio::Behaviour as Mdns, Event as MdnsEvent},
    quic::{tokio::Transport as QuicTransport, Config as QuicConfig},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent, Config as SwarmConfig},
    PeerId, Transport,
};

use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};
use anyhow::Result;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum MeshEvent {
    Gossipsub(GossipsubEvent),
    Mdns(MdnsEvent),
}

impl From<MdnsEvent> for MeshEvent {
    fn from(event: MdnsEvent) -> Self {
        MeshEvent::Mdns(event)
    }
}

impl From<GossipsubEvent> for MeshEvent {
    fn from(event: GossipsubEvent) -> Self {
        MeshEvent::Gossipsub(event)
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "MeshEvent")]
pub struct MeshBehaviour {
    #[behaviour(event_process = false)]
    pub gossipsub: Gossipsub,
    #[behaviour(event_process = false)]
    pub mdns: Mdns,
}

pub async fn run_discovery(keypair: Keypair) -> Result<()> {
    let local_peer_id = PeerId::from(keypair.public());

    let transport = QuicTransport::new(QuicConfig::new(&keypair))
        .map(|(peer_id, conn), _| {
            use libp2p::core::muxing::StreamMuxerBox;
            (peer_id, StreamMuxerBox::new(conn))
        })
        .boxed();

    let gossipsub_config = GossipsubConfigBuilder::default().build()?;
    let mut gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    ).expect("Failed to create gossipsub");
    let topic = IdentTopic::new("cortex/announce");
    gossipsub.subscribe(&topic)?;

    let mdns = Mdns::new(Default::default(), local_peer_id)?;
    let behaviour = MeshBehaviour { gossipsub, mdns };

    let config = SwarmConfig::with_tokio_executor();
    let mut swarm = Swarm::new(transport, behaviour, local_peer_id, config);
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

    println!("Peer ID: {}", local_peer_id);
    println!("Listening for connections...");

    let registry = Arc::new(Mutex::new(Registry::default()));

    // Publication de l'état local
    let my_shards = vec!["layer_0/mlp", "layer_0/attn"]
        .into_iter().map(String::from).collect();

    let announce = AnnounceMsg {
        node_id: local_peer_id.to_string(),
        shards: my_shards,
        version: "v1.0.0".into(),
        vram_free_mb: 2048,
    };

    let json = serde_json::to_vec(&announce)?;
    swarm.behaviour_mut().gossipsub.publish(topic.clone(), json)?;

    // Spawn affichage régulier du Registry
    let reg_clone = Arc::clone(&registry);
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await;
            if let Ok(r) = reg_clone.lock() {
                println!("\n📦 Registry Snapshot:");
                println!("{}", r.snapshot_json());
            }
        }
    });

    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::Behaviour(MeshEvent::Gossipsub(GossipsubEvent::Message { message, .. })) => {
                if let Ok(msg) = serde_json::from_slice::<AnnounceMsg>(&message.data) {
                    if msg.node_id != local_peer_id.to_string() {
                        if let Ok(mut reg) = registry.lock() {
                            reg.update_from_announce(msg);
                        }
                    }
                }
            }

            SwarmEvent::Behaviour(MeshEvent::Mdns(MdnsEvent::Discovered(peers))) => {
                for (peer_id, addr) in peers {
                    println!("Discovered peer: {} at {}", peer_id, addr);
                }
            }

            SwarmEvent::Behaviour(MeshEvent::Mdns(MdnsEvent::Expired(peers))) => {
                for (peer_id, addr) in peers {
                    println!("Peer expired: {} at {}", peer_id, addr);
                }
            }

            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on: {}", address);
            }

            _ => {}
        }
    }

    Ok(())
}
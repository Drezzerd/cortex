use crate::registry::{AnnounceMsg, Registry};

use libp2p::{
    gossipsub::{
        Behaviour as Gossipsub,
        ConfigBuilder as GossipsubConfigBuilder,
        Event as GossipsubEvent,
        IdentTopic,
        MessageAuthenticity,
    },
    identity::Keypair,
    kad::{
        store::MemoryStore,
        Behaviour as Kademlia,
        Config as KademliaConfig,
        Event as KademliaEvent,
        RecordKey,
    },
    mdns::{tokio::Behaviour as Mdns, Event as MdnsEvent},
    multiaddr::{Multiaddr, Protocol},
    quic::{tokio::Transport as QuicTransport, Config as QuicConfig},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent, Config as SwarmConfig},
    core::muxing::StreamMuxerBox,
    PeerId, Transport,
};

use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};
use anyhow::Result;
use std::sync::{Arc, Mutex};

const CORTEX_SHARED_KEY: &str = "cortex-mesh:v1";

#[derive(Debug)]
pub enum MeshEvent {
    Gossipsub(GossipsubEvent),
    Mdns(MdnsEvent),
    Kad(KademliaEvent),
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

impl From<KademliaEvent> for MeshEvent {
    fn from(event: KademliaEvent) -> Self {
        MeshEvent::Kad(event)
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "MeshEvent", event_process = false, out_event = "MeshEvent")]
pub struct MeshBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    pub kad: Kademlia<MemoryStore>,
}

pub async fn run_discovery(keypair: Keypair) -> Result<()> {
    let local_peer_id = PeerId::from(keypair.public());

    let transport = QuicTransport::new(QuicConfig::new(&keypair))
        .map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn)))
        .boxed();

    let gossipsub_config = GossipsubConfigBuilder::default().build()?;
    let mut gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    ).expect("Failed to create gossipsub");
    let topic = IdentTopic::new("cortex/announce");
    gossipsub.subscribe(&topic)?;

    let mdns = Mdns::new(Default::default(), local_peer_id)?;

    let store = MemoryStore::new(local_peer_id);
    let mut kad = Kademlia::with_config(local_peer_id, store, KademliaConfig::default());

    let discovery_key = RecordKey::new(&CORTEX_SHARED_KEY);
    kad.start_providing(discovery_key.clone())?;
    kad.get_providers(discovery_key.clone());

    if let Ok(seed) = std::env::var("CORTEX_BOOTSTRAP_PEER") {
        if let Ok(addr) = seed.parse::<Multiaddr>() {
            if let Some(Protocol::P2p(multihash)) = addr.iter().last() {
                if let Ok(peer_id) = PeerId::from_multihash(multihash.clone().into()) {
                    println!("🌐 Ajout du noeud bootstrap sécurisé : {} @ {}", peer_id, addr);
                    kad.add_address(&peer_id, addr);
                }
            }
        }
    }

    let behaviour = MeshBehaviour { gossipsub, mdns, kad };

    let config = SwarmConfig::with_tokio_executor();
    let mut swarm = Swarm::new(transport, behaviour, local_peer_id, config);
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

    println!("Peer ID: {}", local_peer_id);
    println!("Listening for connections...");

    let registry = Arc::new(Mutex::new(Registry::default()));

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
            },
            SwarmEvent::Behaviour(MeshEvent::Mdns(MdnsEvent::Discovered(peers))) => {
                for (peer_id, addr) in peers {
                    println!("Discovered peer: {} at {}", peer_id, addr);
                    swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                }
            },
            SwarmEvent::Behaviour(MeshEvent::Mdns(MdnsEvent::Expired(peers))) => {
                for (peer_id, addr) in peers {
                    println!("Peer expired: {} at {}", peer_id, addr);
                }
            },
            SwarmEvent::Behaviour(MeshEvent::Kad(event)) => {
                println!("🔁 Kademlia event: {:?}", event);
            },
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on: {}", address);
            },
            _ => {}
        }
    }

    Ok(())
}

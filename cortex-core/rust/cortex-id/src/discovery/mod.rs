use crate::registry::{AnnounceMsg, Registry};

use libp2p::{
    // Updated imports for gossipsub
    gossipsub::{self, Gossipsub, GossipsubEvent, IdentTopic, MessageAuthenticity, ValidationMode},
    identity::Keypair,
    // Updated imports for kad
    kad::{self, store::MemoryStore, Kademlia, KademliaConfig, KademliaEvent},
    // Updated imports for mdns
    mdns::{self, Mdns, MdnsConfig, MdnsEvent},
    multiaddr::{Multiaddr, Protocol},
    quic,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    PeerId,
};

use tokio::time::{sleep, Duration};
use anyhow::Result;
use std::sync::{Arc, Mutex};

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
#[behaviour(out_event = "MeshEvent")]
pub struct MeshBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    pub kad: Kademlia<MemoryStore>,
}

pub async fn run_discovery(keypair: Keypair) -> Result<()> {
    let local_peer_id = PeerId::from(keypair.public());

    // Updated for compatibility with libp2p 0.53
    let transport = quic::tokio::Transport::new(quic::Config::new(&keypair));
    
    // Création du comportement Gossipsub
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .validation_mode(ValidationMode::Strict)
        .build()
        .expect("Valid config");
        
    let mut gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    )?;
    
    let topic = IdentTopic::new("cortex/announce");
    gossipsub.subscribe(&topic)?;

    // Mise à jour pour mdns
    let mdns = Mdns::new(MdnsConfig::default(), local_peer_id)?;

    // Configuration de Kademlia
    let store = MemoryStore::new(local_peer_id);
    let kad_config = KademliaConfig::default();
    let mut kad = Kademlia::with_config(local_peer_id, store, kad_config);

    // Ajout d'un nœud bootstrap si configuré
    if let Ok(seed) = std::env::var("CORTEX_BOOTSTRAP_PEER") {
        if let Ok(addr) = seed.parse::<Multiaddr>() {
            if let Some(Protocol::P2p(multihash)) = addr.iter().last() {
                // Fixed: using from_multihash instead of try_from_multihash
                if let Ok(peer_id) = PeerId::from_multihash(multihash) {
                    println!("🌐 Ajout du noeud bootstrap sécurisé : {} @ {}", peer_id, addr);
                    kad.add_address(&peer_id, addr);
                }
            }
        }
    }

    let behaviour = MeshBehaviour { gossipsub, mdns, kad };

    // Updated for libp2p 0.53: using Swarm::new instead of with_tokio_transport
    let mut swarm = Swarm::new(
        transport,
        behaviour,
        local_peer_id,
        Swarm::config(),
    );
    
    // Écoute sur toutes les interfaces
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
    
    // Publiez le message initial
    swarm.behaviour_mut().gossipsub.publish(topic.clone(), json)?;

    // Configuration des snapshots périodiques
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

    // Boucle principale de traitement des événements
    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::Behaviour(MeshEvent::Gossipsub(GossipsubEvent::Message { 
                propagation_source: _,
                message_id: _,
                message,
            })) => {
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
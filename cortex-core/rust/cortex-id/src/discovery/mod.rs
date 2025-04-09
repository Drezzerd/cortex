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
        QueryResult,
        RecordKey,
    },
    mdns::{tokio::Behaviour as Mdns, Event as MdnsEvent},
    multiaddr::{Multiaddr, Protocol},
    quic::{tokio::Transport as QuicTransport, Config as QuicConfig},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent, Config as SwarmConfig},
    core::muxing::StreamMuxerBox,
    PeerId, Transport,
};

// Change this import to use futures from libp2p
use libp2p::futures::StreamExt;
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
    // Ne pas faire l'appel immédiat à kad.get_providers ici

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

    // FIX 1: Instead of sharing swarm with a mutex, use channels
    let registry_clone = Arc::clone(&registry);
    let discovery_key_clone = discovery_key.clone();
    
    // Spawn a separate task that sends DHT search commands through a channel
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<()>(10);
    let cmd_tx_clone = cmd_tx.clone();
    
    // Task for triggering DHT searches
    tokio::spawn(async move {
        // Attendre que mDNS ait une chance de découvrir des pairs
        sleep(Duration::from_secs(5)).await;
        
        let mut attempts = 0;
        let max_attempts = 5;
        
        while attempts < max_attempts {
            println!("🔍 Recherche de fournisseurs DHT, tentative {}/{}", attempts + 1, max_attempts);
            // Send command to perform DHT search
            if let Err(e) = cmd_tx_clone.send(()).await {
                println!("Error sending DHT search command: {}", e);
            }
            attempts += 1;
            sleep(Duration::from_secs(10)).await;
        }
    });

    // Spawn task for periodic registry snapshots
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

    // Main event loop
    loop {
        tokio::select! {
            // Process any commands received to trigger DHT searches
            Some(_) = cmd_rx.recv() => {
                swarm.behaviour_mut().kad.get_providers(discovery_key_clone.clone());
            }
            
            // Process swarm events
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(MeshEvent::Gossipsub(GossipsubEvent::Message { message, .. })) => {
                        if let Ok(msg) = serde_json::from_slice::<AnnounceMsg>(&message.data) {
                            if msg.node_id != local_peer_id.to_string() {
                                if let Ok(mut reg) = registry_clone.lock() {
                                    reg.update_from_announce(msg);
                                }
                            }
                        }
                    },
                    SwarmEvent::Behaviour(MeshEvent::Mdns(MdnsEvent::Discovered(peers))) => {
                        for (peer_id, addr) in peers {
                            println!("🔍 Discovered peer: {} at {}", peer_id, addr);
                            swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                        }
                    },
                    SwarmEvent::Behaviour(MeshEvent::Mdns(MdnsEvent::Expired(peers))) => {
                        for (peer_id, addr) in peers {
                            println!("⚠️ Peer expired: {} at {}", peer_id, addr);
                        }
                    },
                    SwarmEvent::Behaviour(MeshEvent::Kad(KademliaEvent::RoutingUpdated { peer, .. })) => {
                        println!("✅ Routing table updated with peer: {}", peer);
                    },
                    SwarmEvent::Behaviour(MeshEvent::Kad(KademliaEvent::UnroutablePeer { peer })) => {
                        println!("⚠️ Unroutable peer: {}", peer);
                    },
                    // FIX 2: Use proper pattern matching for QueryResult
                    SwarmEvent::Behaviour(MeshEvent::Kad(KademliaEvent::OutboundQueryProgressed { result, ..})) => {
                        match result {
                            QueryResult::GetProviders(Ok(_)) => {
                                println!("✅ DHT GetProviders query completed successfully");
                            },
                            QueryResult::StartProviding(Ok(_)) => {
                                println!("✅ DHT StartProviding query completed successfully");
                            },
                            QueryResult::GetProviders(Err(e)) => {
                                println!("⚠️ DHT GetProviders query failed: {:?}", e);
                            },
                            QueryResult::StartProviding(Err(e)) => {
                                println!("⚠️ DHT StartProviding query failed: {:?}", e);
                            },
                            _ => println!("✅ Other DHT query completed: {:?}", result),
                        }
                    },
                    SwarmEvent::Behaviour(MeshEvent::Kad(event)) => {
                        println!("🔁 Kademlia event: {:?}", event);
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("🔊 Listening on: {}", address);
                    },
                    _ => {}
                }
            }
        }
    }
}
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
use libp2p::futures::StreamExt;
use tokio::time::{sleep, Duration};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::str::FromStr;

const CORTEX_SHARED_KEY: &[u8; 14] = b"cortex-mesh:v1";
const ANNOUNCE_TOPIC: &str = "cortex/announce";
const BOOTSTRAP_INTERVAL: u64 = 30; // secondes

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
#[behaviour(to_swarm = "MeshEvent", event_process = false)]
pub struct MeshBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    pub kad: Kademlia<MemoryStore>,
}

#[derive(Debug)]
enum Command {
    GetProviders,
    AnnounceNode,
}

/// Fonction utilitaire pour convertir une chaîne bootstrap en multiaddr et peer_id
fn parse_bootstrap_addr(addr_str: &str) -> Option<(Multiaddr, PeerId)> {
    match Multiaddr::from_str(addr_str) {
        Ok(addr) => {
            // Extraire le PeerId de la multiaddr
            for protocol in addr.iter() {
                if let Protocol::P2p(multihash) = protocol {
                    if let Ok(peer_id) = PeerId::from_multihash(multihash) {
                        return Some((addr, peer_id));
                    }
                }
            }
            println!("❌ Pas de PeerId trouvé dans l'adresse: {}", addr_str);
            None
        },
        Err(e) => {
            println!("❌ Impossible de parser l'adresse bootstrap {}: {}", addr_str, e);
            None
        }
    }
}

/// Crée un transport QUIC commun pour tous les nœuds
fn create_transport(keypair: &Keypair) -> libp2p::core::transport::Boxed<(PeerId, StreamMuxerBox)> {
    QuicTransport::new(QuicConfig::new(keypair))
        .map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn)))
        .boxed()
}

/// Construit le comportement mesh de base (commun à tous les nœuds)
async fn build_mesh_behaviour(keypair: Keypair, local_peer_id: PeerId) -> Result<MeshBehaviour> {
    // Configuration de Gossipsub améliorée
    let gossipsub_config = GossipsubConfigBuilder::default()
        .flood_publish(true)
        .build()?;
    
    let mut gossipsub = Gossipsub::new(MessageAuthenticity::Signed(keypair.clone()), gossipsub_config)
        .expect("Échec de création de gossipsub");
    
    let topic = IdentTopic::new(ANNOUNCE_TOPIC);
    gossipsub.subscribe(&topic)?;
    
    // mDNS pour découverte locale (LAN)
    let mdns = Mdns::new(Default::default(), local_peer_id)?;
    
    // Kademlia pour DHT
    let store = MemoryStore::new(local_peer_id);
    let kad_config = KademliaConfig::default();
    let kad = Kademlia::with_config(local_peer_id, store, kad_config);
    
    Ok(MeshBehaviour { gossipsub, mdns, kad })
}

/// Lancement d'un nœud bootstrap qui reste en écoute même en l'absence de pairs.
pub async fn run_bootstrap_node(keypair: Keypair) -> Result<()> {
    let local_peer_id = PeerId::from(keypair.public());
    println!("🌐 Nœud bootstrap avec PeerId: {}", local_peer_id);
    
    // Création du transport
    let transport = create_transport(&keypair);
    
    // Construction du comportement
    let mut behaviour = build_mesh_behaviour(keypair.clone(), local_peer_id).await?;
    
    // Configuration spécifique bootstrap: démarrer en tant que fournisseur DHT
    let discovery_key = RecordKey::new(&CORTEX_SHARED_KEY);
    match behaviour.kad.start_providing(discovery_key.clone()) {
        Ok(query_id) => println!("✅ DHT StartProviding lancé avec succès, QueryId: {:?}", query_id),
        Err(e) => println!("⚠️ Échec de DHT StartProviding : {:?}. Continuité en mode bootstrap.", e),
    }
    
    // Configuration et démarrage du swarm
    let config = SwarmConfig::with_tokio_executor();
    let mut swarm = Swarm::new(transport, behaviour, local_peer_id, config);
    
    // Écoute sur tous les interfaces (port aléatoire UDP pour QUIC)
    for addr in ["/ip4/0.0.0.0/udp/0/quic-v1", "/ip6/::/udp/0/quic-v1"] {
        match swarm.listen_on(addr.parse()?) {
            Ok(_) => println!("Écoute démarrée sur {}", addr),
            Err(e) => println!("⚠️ Impossible d'écouter sur {}: {}", addr, e),
        }
    }
    
    // Registre partagé
    let registry = Arc::new(Mutex::new(Registry::default()));
    
    // Canal pour les commandes planifiées
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<Command>(10);
    
    // Tâche pour DHT bootstrap périodique
    let cmd_tx_clone = cmd_tx.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(BOOTSTRAP_INTERVAL)).await;
            println!("🔍 Recherche DHT pour les fournisseurs...");
            if let Err(e) = cmd_tx_clone.send(Command::GetProviders).await {
                println!("Erreur lors de l'envoi de la commande DHT: {:?}", e);
            }
        }
    });
    
    // Tâche pour annonce périodique
    let cmd_tx_clone = cmd_tx.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(45)).await;
            if let Err(e) = cmd_tx_clone.send(Command::AnnounceNode).await {
                println!("Erreur lors de l'envoi de la commande d'annonce: {:?}", e);
            }
        }
    });
    
    // Tâche pour afficher le registre
    let reg_clone = Arc::clone(&registry);
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(60)).await;
            if let Ok(r) = reg_clone.lock() {
                println!("\n📊 Registry Snapshot:");
                println!("{}", r.snapshot_json());
            }
        }
    });
    
    // Attendre que les addresses d'écoute soient établies
    let mut listening = false;
    while !listening {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm.select_next_some().await {
            println!("📡 Bootstrap en écoute sur: {}", address);
            listening = true;
        }
    }
    
    // Boucle principale
    let topic = IdentTopic::new(ANNOUNCE_TOPIC);
    let discovery_key_clone = discovery_key.clone();
    
    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    Command::GetProviders => {
                        swarm.behaviour_mut().kad.get_providers(discovery_key_clone.clone());
                    },
                    Command::AnnounceNode => {
                        // Création d'un message d'annonce
                        let announce = AnnounceMsg {
                            node_id: local_peer_id.to_string(),
                            shards: vec!["bootstrap".into()],
                            version: env!("CARGO_PKG_VERSION").to_string(),
                            vram_free_mb: 0,
                        };
                        
                        if let Ok(data) = serde_json::to_vec(&announce) {
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), data) {
                                println!("⚠️ Erreur lors de l'annonce: {:?}", e);
                            } else {
                                println!("📢 Annonce publiée sur {}", topic);
                            }
                        }
                    }
                }
            },
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(MeshEvent::Gossipsub(GossipsubEvent::Message { message, .. })) => {
                        if let Ok(msg) = serde_json::from_slice::<AnnounceMsg>(&message.data) {
                            if msg.node_id != local_peer_id.to_string() {
                                println!("📨 Message reçu de: {}", msg.node_id);
                                if let Ok(mut reg) = registry.lock() {
                                    reg.update_from_announce(msg);
                                }
                            }
                        }
                    },
                    SwarmEvent::Behaviour(MeshEvent::Mdns(MdnsEvent::Discovered(peers))) => {
                        for (peer_id, addr) in peers {
                            println!("🔍 Pair découvert via mDNS: {} à {}", peer_id, addr);
                            swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                        }
                    },
                    SwarmEvent::Behaviour(MeshEvent::Kad(KademliaEvent::RoutingUpdated { peer, .. })) => {
                        println!("📝 Table de routage mise à jour avec: {}", peer);
                    },
                    SwarmEvent::Behaviour(MeshEvent::Kad(KademliaEvent::OutboundQueryProgressed { result, .. })) => {
                        println!("📊 Progression requête DHT: {:?}", result);
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("📡 En écoute sur: {}", address);
                    },
                    _ => {}
                }
            }
        }
    }
}

/// Fonction pour lancer un nœud "léger" qui rejoint le réseau
pub async fn run_light_node(keypair: Keypair) -> Result<()> {
    let local_peer_id = PeerId::from(keypair.public());
    println!("🔹 Nœud léger avec PeerId: {}", local_peer_id);
    
    // Création du transport
    let transport = create_transport(&keypair);
    
    // Construction du comportement
    let behaviour = build_mesh_behaviour(keypair.clone(), local_peer_id).await?;
    
    // Configuration et démarrage du swarm
    let config = SwarmConfig::with_tokio_executor();
    let mut swarm = Swarm::new(transport, behaviour, local_peer_id, config);
    
    // Écoute sur tous les interfaces
    for addr in ["/ip4/0.0.0.0/udp/0/quic-v1", "/ip6/::/udp/0/quic-v1"] {
        match swarm.listen_on(addr.parse()?) {
            Ok(_) => println!("Écoute démarrée sur {}", addr),
            Err(e) => println!("⚠️ Impossible d'écouter sur {}: {}", addr, e),
        }
    }
    
    // Ajout du nœud bootstrap si spécifié
    if let Ok(bootstrap_addr) = std::env::var("CORTEX_BOOTSTRAP_PEER") {
        println!("🔌 Bootstrap avec: {}", bootstrap_addr);
        
        if let Some((addr, peer_id)) = parse_bootstrap_addr(&bootstrap_addr) {
            println!("🌐 Connexion au nœud bootstrap: {} @ {}", peer_id, addr);
            swarm.behaviour_mut().kad.add_address(&peer_id, addr.clone());
            
            // Tentative de connexion directe
            match swarm.dial(addr.clone()) {
                Ok(_) => println!("✅ Tentative de connexion à {}", addr),
                Err(e) => println!("❌ Échec de connexion à {}: {:?}", addr, e),
            }
        } else {
            println!("❌ Format d'adresse bootstrap invalide");
        }
    } else {
        println!("⚠️ Aucun nœud bootstrap spécifié. Utilisation de mDNS uniquement.");
    }
    
    // Mise en place des clés DHT
    let discovery_key = RecordKey::new(&CORTEX_SHARED_KEY);
    
    // Canal pour les commandes
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<Command>(10);
    
    // Tâche pour recherche DHT périodique
    let cmd_tx_clone = cmd_tx.clone();
    tokio::spawn(async move {
        // Attente initiale pour laisser le réseau s'établir
        sleep(Duration::from_secs(2)).await;
        
        loop {
            if let Err(e) = cmd_tx_clone.send(Command::GetProviders).await {
                println!("Erreur lors de l'envoi de la commande DHT: {:?}", e);
            }
            
            // Annonce après 5 secondes
            sleep(Duration::from_secs(5)).await;
            
            if let Err(e) = cmd_tx_clone.send(Command::AnnounceNode).await {
                println!("Erreur lors de l'envoi de la commande d'annonce: {:?}", e);
            }
            
            sleep(Duration::from_secs(BOOTSTRAP_INTERVAL)).await;
        }
    });
    
    // Attendre que les addresses d'écoute soient établies
    let mut listening = false;
    while !listening {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm.select_next_some().await {
            println!("📡 Nœud léger en écoute sur: {}", address);
            listening = true;
        }
    }
    
    // Boucle principale
    let topic = IdentTopic::new(ANNOUNCE_TOPIC);
    let discovery_key_clone = discovery_key.clone();
    
    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    Command::GetProviders => {
                        println!("🔍 Recherche de fournisseurs pour la clé: {:?}", discovery_key_clone);
                        swarm.behaviour_mut().kad.get_providers(discovery_key_clone.clone());
                    },
                    Command::AnnounceNode => {
                        // Création d'un message d'annonce
                        let announce = AnnounceMsg {
                            node_id: local_peer_id.to_string(),
                            shards: vec!["light".into()],
                            version: env!("CARGO_PKG_VERSION").to_string(),
                            vram_free_mb: 0,
                        };
                        
                        if let Ok(data) = serde_json::to_vec(&announce) {
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), data) {
                                println!("⚠️ Erreur lors de l'annonce: {:?}", e);
                            } else {
                                println!("📢 Annonce publiée sur {}", topic);
                            }
                        }
                    }
                }
            },
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(MeshEvent::Gossipsub(GossipsubEvent::Message { message, .. })) => {
                        println!("📨 Message gossipsub reçu");
                        if let Ok(msg) = serde_json::from_slice::<AnnounceMsg>(&message.data) {
                            println!("📨 Annonce reçue de: {}", msg.node_id);
                        }
                    },
                    SwarmEvent::Behaviour(MeshEvent::Mdns(MdnsEvent::Discovered(peers))) => {
                        for (peer_id, addr) in peers {
                            println!("🔍 Pair découvert via mDNS: {} à {}", peer_id, addr);
                            swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                        }
                    },
                    SwarmEvent::Behaviour(MeshEvent::Kad(KademliaEvent::RoutingUpdated { peer, .. })) => {
                        println!("📝 Table de routage mise à jour avec: {}", peer);
                    },
                    SwarmEvent::Behaviour(MeshEvent::Kad(KademliaEvent::OutboundQueryProgressed { result, .. })) => {
                        println!("📊 Progression requête DHT: {:?}", result);
                    },
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("🔗 Connexion établie avec: {}", peer_id);
                    },
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        println!("❌ Connexion fermée avec: {}", peer_id);
                    },
                    _ => {}
                }
            }
        }
    }
}
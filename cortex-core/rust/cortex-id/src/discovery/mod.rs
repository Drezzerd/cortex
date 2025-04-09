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
#[behaviour(to_swarm = "MeshEvent", event_process = false)]
pub struct MeshBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    pub kad: Kademlia<MemoryStore>,
}

#[derive(Debug)]
enum Command {
    GetProviders,
}

/// Lancement d'un nœud bootstrap qui reste en écoute même en l'absence de pairs.
/// La recherche DHT est déclenchée périodiquement via un canal.
pub async fn run_bootstrap_node(keypair: Keypair) -> Result<()> {
    let local_peer_id = PeerId::from(keypair.public());

    // Création du transport QUIC
    let transport = QuicTransport::new(QuicConfig::new(&keypair))
        .map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn)))
        .boxed();

    // Initialisation de Gossipsub
    let gossipsub_config = GossipsubConfigBuilder::default().build()?;
    let mut gossipsub = Gossipsub::new(MessageAuthenticity::Signed(keypair.clone()), gossipsub_config)
        .expect("Échec de création de gossipsub");
    let topic = IdentTopic::new("cortex/announce");
    gossipsub.subscribe(&topic)?;

    // Démarrage de mDNS (optionnel pour la découverte locale)
    let mdns = Mdns::new(Default::default(), local_peer_id)?;

    // Mise en place de Kademlia avec un store en mémoire
    let store = MemoryStore::new(local_peer_id);
    let mut kad = Kademlia::with_config(local_peer_id, store, KademliaConfig::default());
    let discovery_key = RecordKey::new(&CORTEX_SHARED_KEY);

    // Lancement du fournisseur DHT avec gestion de la valeur de retour (QueryId)
    match kad.start_providing(discovery_key.clone()) {
        Ok(query_id) => println!("DHT StartProviding lancé avec succès, QueryId: {:?}", query_id),
        Err(e) => println!("⚠️ Échec de DHT StartProviding : {:?}. Continuité en mode bootstrap.", e),
    }

    // Ajout d'un peer bootstrap si défini via la variable d'environnement
    if let Ok(seed) = std::env::var("CORTEX_BOOTSTRAP_PEER") {
        if let Ok(addr) = seed.parse::<Multiaddr>() {
            if let Some(Protocol::P2p(multihash)) = addr.iter().last() {
                if let Ok(peer_id) = PeerId::from_multihash(multihash.clone().into()) {
                    println!("🌐 Ajout du nœud bootstrap sécurisé : {} @ {}", peer_id, addr);
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
    println!("Noeud bootstrap en écoute...");

    let registry = Arc::new(Mutex::new(Registry::default()));

    // Création d'un canal pour envoyer des commandes DHT à la boucle principale.
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<Command>(10);

    // Tâche qui envoie périodiquement une commande GetProviders via le canal.
    let cmd_tx_clone = cmd_tx.clone();
    tokio::spawn(async move {
        loop {
            println!("🔍 Lancement d'une recherche DHT pour les fournisseurs...");
            if let Err(e) = cmd_tx_clone.send(Command::GetProviders).await {
                println!("Erreur lors de l'envoi de la commande DHT: {:?}", e);
            }
            sleep(Duration::from_secs(10)).await;
        }
    });

    // Tâche pour afficher périodiquement un snapshot du registre
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

    // Boucle principale de traitement des événements du swarm et des commandes
    let discovery_key_clone = discovery_key.clone();
    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    Command::GetProviders => {
                        swarm.behaviour_mut().kad.get_providers(discovery_key_clone.clone());
                    }
                }
            },
            event = swarm.select_next_some() => {
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
                            println!("🔍 Pair découvert : {} à {}", peer_id, addr);
                            swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                        }
                    },
                    SwarmEvent::Behaviour(MeshEvent::Kad(KademliaEvent::RoutingUpdated { peer, .. })) => {
                        println!("✅ Table de routage mise à jour avec: {}", peer);
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("🔊 En écoute sur: {}", address);
                    },
                    _ => {}
                }
            }
        }
    }
}

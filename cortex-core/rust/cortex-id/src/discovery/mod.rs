// ----- discovery/mod.rs -----
use libp2p::{
    gossipsub::{
        Behaviour as Gossipsub, ConfigBuilder as GossipsubConfigBuilder, Event as GossipsubEvent, MessageAuthenticity,
    },
    identity::Keypair,
    mdns::{tokio::Behaviour as Mdns, Event as MdnsEvent},
    noise,
    quic::{tokio::Transport as QuicTransport, Config as QuicConfig},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent, Config as SwarmConfig},
    PeerId, Transport,
};

use tokio_stream::StreamExt;
use anyhow::Result;

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
    let peer_id = PeerId::from(keypair.public());

    // Configuration de base pour quic
    let transport = QuicTransport::new(QuicConfig::new(&keypair));

    // Convertir en StreamMuxerBox pour le muxing 
    let transport = transport.map(|(peer_id, conn), _| {
        // Utiliser l'API correcte pour convertir une connexion QuicTransport en StreamMuxerBox
        use libp2p::core::muxing::StreamMuxerBox;
        use libp2p::core::upgrade::Version;
        use std::time::Duration;
        
        (peer_id, StreamMuxerBox::new(conn, Version::V1))
    }).boxed();

    // Configuration de gossipsub
    let gossipsub_config = GossipsubConfigBuilder::default().build().expect("Failed to build gossipsub config");
    let gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    ).expect("Failed to create gossipsub");

    let mdns = Mdns::new(Default::default(), peer_id).expect("Failed to create mDNS");
    let behaviour = MeshBehaviour { gossipsub, mdns };

    // Créer une config SwarmConfig explicitemente 
    let config = SwarmConfig::new();
    
    // Création du swarm avec la config explicite
    let mut swarm = Swarm::new(transport, behaviour, peer_id, config);

    // Écouter sur une adresse locale
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

    println!("Peer ID: {}", peer_id);
    println!("Listening for connections...");

    // Boucle d'événements
    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::Behaviour(MeshEvent::Gossipsub(e)) => {
                println!("Gossipsub event: {:?}", e);
            }
            SwarmEvent::Behaviour(MeshEvent::Mdns(e)) => {
                match e {
                    MdnsEvent::Discovered(peers) => {
                        for (peer_id, addr) in peers {
                            println!("Discovered peer: {} at {}", peer_id, addr);
                            // Vous pourriez essayer de vous connecter ici si nécessaire
                            // swarm.dial(peer_id)?;
                        }
                    }
                    MdnsEvent::Expired(peers) => {
                        for (peer_id, addr) in peers {
                            println!("Peer expired: {} at {}", peer_id, addr);
                        }
                    }
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
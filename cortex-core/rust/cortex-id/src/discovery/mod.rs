// ----- discovery/mod.rs -----
use libp2p::{
    gossipsub::{
        Behaviour as Gossipsub, ConfigBuilder as GossipsubConfigBuilder, Event as GossipsubEvent, MessageAuthenticity,
    },
    identity::Keypair,
    mdns::{tokio::Behaviour as Mdns, Event as MdnsEvent},
    noise,
    quic::{tokio::Transport as QuicTransport, Config as QuicConfig},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
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

    // Création de la configuration noise (utilise l'API correcte)
    let noise_config = noise::Config::new(&keypair).expect("Failed to create noise config");
    let transport = QuicTransport::new(QuicConfig::new(&keypair)).boxed();

    // Configuration de gossipsub
    let gossipsub_config = GossipsubConfigBuilder::default().build().expect("Failed to build gossipsub config");
    let gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    ).expect("Failed to create gossipsub");

    let mdns = Mdns::new(Default::default(), peer_id).expect("Failed to create mDNS");
    let behaviour = MeshBehaviour { gossipsub, mdns };

    // Création du swarm avec l'API correcte
    let mut swarm = Swarm::new(transport, behaviour, peer_id);

    // Boucle d'événements
    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::Behaviour(MeshEvent::Gossipsub(e)) => {
                println!("Gossipsub event: {:?}", e);
            }
            SwarmEvent::Behaviour(MeshEvent::Mdns(e)) => {
                println!("mDNS event: {:?}", e);
            }
            _ => {}
        }
    }

    Ok(())
}
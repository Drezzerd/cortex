use libp2p::{
    gossipsub::{
        Behaviour as Gossipsub, Config as GossipsubConfig, Event as GossipsubEvent, MessageAuthenticity,
    },
    identity::Keypair,
    mdns::{tokio::Behaviour as Mdns, Event as MdnsEvent},
    noise::Config as NoiseConfig,
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

    let noise_config = NoiseConfig::new(&keypair)?;
    let transport = QuicTransport::new(QuicConfig::new(&keypair)).boxed();

    let gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(keypair.clone()),
        GossipsubConfig::default(),
    )?;

    let mdns = Mdns::new(Default::default(), peer_id)?;
    let behaviour = MeshBehaviour { gossipsub, mdns };

    let mut swarm = Swarm::with_tokio_executor(transport, behaviour, peer_id);

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
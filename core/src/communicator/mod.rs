use libp2p::gossipsub::{
    Behaviour as Gossipsub,
    Config as GossipsubConfig,
    Event as GossipsubEvent,
    IdentTopic,
    MessageAuthenticity,
};
use libp2p::identity::Keypair;
use libp2p::PeerId;
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Message standard pour la communication entre nœuds
#[derive(Debug, Serialize, Deserialize)]
pub struct CommunicatorMessage {
    pub sender: String,
    pub payload: String,
    pub timestamp: u64,
}

/// Le Communicator encapsule la logique d’envoi et de réception de messages
pub struct Communicator {
    pub gossipsub: Gossipsub,
    pub topic: IdentTopic,
}

impl Communicator {
    /// Crée un nouveau Communicator en initialisant gossipsub avec la clé et
    /// en s’abonnant au topic dédié aux communications.
    pub fn new(keypair: &Keypair) -> Result<Self> {
        let _peer_id = PeerId::from(keypair.public());
        
        // Configuration de Gossipsub via Config (v0.53)
        let gossipsub_config = GossipsubConfig::default();
        let mut gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config
        ).expect("Erreur lors de la création de Gossipsub");
        
        // Définir un topic pour la communication (par exemple "cortex/communicator")
        let topic = IdentTopic::new("cortex/communicator");
        gossipsub.subscribe(&topic)?;
        
        Ok(Communicator { gossipsub, topic })
    }

    /// Envoie un message via Gossipsub.
    /// La fonction sérialise le message en JSON et le publie sur le topic défini.
    pub fn send_message(&mut self, msg: &CommunicatorMessage) -> Result<()> {
        let json = serde_json::to_vec(msg)?;
        self.gossipsub.publish(self.topic.clone(), json)?;
        Ok(())
    }
    
    /// Traite un événement Gossipsub reçu.
    /// À appeler dans la boucle d’événements du Swarm.
    pub fn handle_event(&self, event: GossipsubEvent) {
        match event {
            GossipsubEvent::Message { message, .. } => {
                match serde_json::from_slice::<CommunicatorMessage>(&message.data) {
                    Ok(comm_msg) => {
                        println!("Message reçu de {} : {}", comm_msg.sender, comm_msg.payload);
                    }
                    Err(e) => eprintln!("Erreur de désérialisation du message: {:?}", e),
                }
            },
            _ => {} // Traiter d'autres types d'événements si nécessaire
        }
    }
}

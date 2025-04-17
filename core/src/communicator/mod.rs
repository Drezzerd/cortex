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
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::registry::{Registry, AnnounceMsg};

pub type SharedCommunicator = Arc<Mutex<Communicator>>;

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
    pub fn handle_event(&mut self, event: GossipsubEvent, registry: Option<Arc<Mutex<Registry>>>) {
        match event {
            GossipsubEvent::Message { message, .. } => {
                println!("Message gossipsub reçu de {:?}, taille: {} octets", 
                        message.source, message.data.len());
                
                // Essayer de désérialiser le message comme message du communicator
                if let Ok(comm_msg) = serde_json::from_slice::<CommunicatorMessage>(&message.data) {
                    println!("Message reçu de {}: {}", comm_msg.sender, comm_msg.payload);
                } 
                // Essayer comme message d'annonce de nœud
                else if let Ok(announce) = serde_json::from_slice::<AnnounceMsg>(&message.data) {
                    println!("Annonce de nœud reçue: {}", announce.node_id);
                    
                    // Si un registry est disponible, mettre à jour le registry
                    if let Some(reg) = registry {
                        let mut reg_lock = match reg.try_lock() {
                            Ok(lock) => lock,
                            Err(_) => {
                                println!("Impossible de verrouiller le registry, annonce ignorée");
                                return;
                            }
                        };
                        reg_lock.update_from_announce(announce);
                        println!("Registry mis à jour avec le nœud");
                    } else {
                        println!("Pas de registry disponible pour l'annonce");
                    }
                } else {
                    println!("Message gossipsub de format inconnu");
                }
            },
            GossipsubEvent::Subscribed { peer_id, topic } => {
                println!("Nœud {:?} abonné au topic {:?}", peer_id, topic);
            },
            GossipsubEvent::Unsubscribed { peer_id, topic } => {
                println!("Nœud {:?} désabonné du topic {:?}", peer_id, topic);
            },
            _ => {} // Autres événements Gossipsub
        }
    }
}
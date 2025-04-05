
// ----- identity/mod.rs -----
use std::path::PathBuf;
use std::fs;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use dirs;
use libp2p::identity::{Keypair, ed25519};
use anyhow::Result;

pub fn load_or_generate_identity() -> Result<Keypair> {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".cortex/identity.key");

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let bytes = STANDARD.decode(content.trim())?;
        
        // Utilisation correcte de l'API ed25519
        let ed25519_keypair = libp2p::identity::ed25519::Keypair::decode(&bytes)
            .map_err(|e| anyhow::anyhow!("Failed to decode keypair: {:?}", e))?;
            
        return Ok(Keypair::Ed25519(ed25519_keypair));
    }

    // Génération d'une nouvelle clé
    let ed25519_keypair = libp2p::identity::ed25519::Keypair::generate();
    
    // Encodage de la clé
    let encoded = STANDARD.encode(ed25519_keypair.encode());
    
    // Sauvegarde de la clé
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, encoded)?;

    Ok(Keypair::Ed25519(ed25519_keypair))
}
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
        
        // Utiliser try_from_bytes au lieu de decode
        let ed25519_keypair = ed25519::Keypair::try_from_bytes(&mut bytes.clone())
            .map_err(|e| anyhow::anyhow!("Failed to decode keypair: {:?}", e))?;
            
        // Utiliser la conversion from() au lieu de Ed25519()
        return Ok(Keypair::from(ed25519_keypair));
    }

    // Génération d'une nouvelle clé
    let ed25519_keypair = ed25519::Keypair::generate();
    
    // Sauvegarde de la clé en utilisant to_bytes()
    let encoded = STANDARD.encode(ed25519_keypair.to_bytes());
    
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, encoded)?;

    // Utiliser from() plutôt que Ed25519()
    Ok(Keypair::from(ed25519_keypair))
}
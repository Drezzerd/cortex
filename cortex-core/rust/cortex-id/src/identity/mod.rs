// ----- identity/mod.rs -----
use std::path::PathBuf;
use std::fs;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use dirs;
use libp2p::identity::{Keypair, ed25519};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityInfo {
    pub peer_id: String,
    pub public_key: String,
}

pub fn load_or_generate_identity() -> Result<Keypair> {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".cortex/identity.key");

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let bytes = STANDARD.decode(content.trim())?;
        
        let ed25519_keypair = ed25519::Keypair::try_from_bytes(&mut bytes.clone())
            .map_err(|e| anyhow::anyhow!("Failed to decode keypair: {:?}", e))?;
            
        return Ok(Keypair::from(ed25519_keypair));
    }

    // Génération d'une nouvelle clé
    let ed25519_keypair = ed25519::Keypair::generate();
    
    // Sauvegarde de la clé en utilisant to_bytes()
    let encoded = STANDARD.encode(ed25519_keypair.to_bytes());
    
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, encoded)?;

    Ok(Keypair::from(ed25519_keypair))
}

pub fn generate_identity() -> Result<IdentityInfo> {
    let keypair = load_or_generate_identity()?;
    let peer_id = PeerId::from(keypair.public());
    
    // Obtenir les bytes de la clé publique pour libp2p 0.53
    let public_key_bytes = match &keypair {
        Keypair::Ed25519(key) => key.public().encode(),
        _ => return Err(anyhow::anyhow!("Unsupported key type")),
    };
    
    let public_key = STANDARD.encode(&public_key_bytes);

    Ok(IdentityInfo {
        peer_id: peer_id.to_string(),
        public_key,
    })
}

pub fn save_identity_file(identity: &IdentityInfo) -> Result<()> {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".cortex/identity.json");

    let json = serde_json::to_string_pretty(identity)
        .context("Failed to serialize identity info")?;

    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, json).context("Failed to write identity file")?;

    Ok(())
}
use std::path::PathBuf;
use std::fs;

use base64::{engine::general_purpose::STANDARD, Engine};
use libp2p::identity::{ed25519, Keypair, PeerId};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityInfo {
    pub peer_id: String,
    pub public_key: String,
}

const KEY_PATH: &str = "/root/.cortex/identity.key";
const INFO_PATH: &str = "/root/.cortex/identity.json";

/// Génère ou charge un ed25519::Keypair brut
pub fn load_or_generate_identity() -> Result<ed25519::Keypair> {
    let path = PathBuf::from(KEY_PATH);

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let bytes = STANDARD.decode(content.trim())?;
        return ed25519::Keypair::try_from_bytes(&mut bytes.clone())
            .map_err(|e| anyhow::anyhow!("Failed to decode keypair: {:?}", e));
    }

    let ed25519_keypair = ed25519::Keypair::generate();
    let encoded = STANDARD.encode(ed25519_keypair.to_bytes());
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, encoded)?;

    Ok(ed25519_keypair)
}

/// Retourne un Keypair utilisable par libp2p
pub fn load_identity_file() -> Result<Keypair> {
    let path = PathBuf::from(KEY_PATH);
    let content = fs::read_to_string(path)?;
    let bytes = STANDARD.decode(content.trim())?;
    let ed25519 = ed25519::Keypair::try_from_bytes(&mut bytes.clone())?;
    Ok(Keypair::from(ed25519))
}

/// Génère ou charge l'identité, et renvoie l'info utilisateur-friendly
pub fn generate_identity() -> Result<IdentityInfo> {
    let ed25519_keypair = load_or_generate_identity()?;
    let public_key_bytes = ed25519_keypair.public().to_bytes();
    let public_key = STANDARD.encode(public_key_bytes);

    let keypair = Keypair::from(ed25519_keypair);
    let peer_id = PeerId::from(keypair.public());

    Ok(IdentityInfo {
        peer_id: peer_id.to_string(),
        public_key,
    })
}

/// Sauvegarde une version lisible de l'identité
pub fn save_identity_file(identity: &IdentityInfo) -> Result<()> {
    let path = PathBuf::from(INFO_PATH);
    let json = serde_json::to_string_pretty(identity)
        .context("Failed to serialize identity info")?;
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, json).context("Failed to write identity file")?;
    Ok(())
}

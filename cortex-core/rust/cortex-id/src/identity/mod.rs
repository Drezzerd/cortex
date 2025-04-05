use std::path::PathBuf;
use std::fs;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use dirs;
use libp2p::identity::{Keypair, ed25519};
use serde::{Deserialize, Serialize};
use anyhow::Result;

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
        let ed25519_keypair = ed25519::Keypair::try_from_bytes(&mut bytes.clone())?;
        return Ok(Keypair::from(ed25519_keypair));
    }

    let ed25519_keypair = ed25519::Keypair::generate();
    let encoded = STANDARD.encode(ed25519_keypair.encode());
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, encoded)?;

    Ok(Keypair::from(ed25519_keypair))
}

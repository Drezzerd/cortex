use libp2p::identity::{Keypair, ed25519};
use libp2p::PeerId;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;
use dirs::home_dir;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Identity {
    pub peer_id: String,
    pub key_base64: String,
}

pub fn generate_identity() -> Identity {
    // On génère une clé ed25519 "raw"
    let ed_kp = ed25519::Keypair::generate();

    // On la convertit en Keypair "générique" de libp2p
    let kp = Keypair::from(ed_kp.clone());

    let encoded = STANDARD.encode(ed_kp.to_bytes());

    Identity {
        peer_id: PeerId::from_public_key(&kp.public()).to_string(),
        key_base64: encoded,
    }
}

pub fn save_identity_file(id: &Identity) -> std::io::Result<()> {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".cortex");
    create_dir_all(&path)?;
    path.push("identity.key");

    let content = serde_json::to_string_pretty(id).unwrap();
    write(path, content)
}

use std::path::PathBuf;
use std::fs;

use base64::{engine::general_purpose::STANDARD, Engine};
use libp2p::identity::{ed25519, Keypair, PeerId};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityInfo {
    pub peer_id: String,
    pub public_key: String,
}

/// Déterminer le chemin du répertoire .cortex en fonction de l'environnement
pub fn get_cortex_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cortex")
    } else if let Some(home) = dirs::home_dir() {
        home.join(".cortex")
    } else {
        // Fallback pour Docker
        PathBuf::from("/home/cortexuser/.cortex")
    }
}

/// Obtenir le chemin du fichier clé
pub fn get_key_path() -> PathBuf {
    get_cortex_dir().join("identity.key")
}

/// Obtenir le chemin du fichier info
pub fn get_info_path() -> PathBuf {
    get_cortex_dir().join("identity.json")
}

/// Génère ou charge un ed25519::Keypair brut
pub fn load_or_generate_identity() -> Result<ed25519::Keypair> {
    let path = get_key_path();
    println!("Chemin de la clé: {:?}", path);

    if path.exists() {
        println!("Chargement de l'identité existante...");
        let content = fs::read_to_string(&path)?;
        let bytes = STANDARD.decode(content.trim())?;
        return ed25519::Keypair::try_from_bytes(&mut bytes.clone())
            .map_err(|e| anyhow!("Failed to decode keypair: {:?}", e));
    }

    println!("Génération d'une nouvelle identité...");
    let ed25519_keypair = ed25519::Keypair::generate();
    let encoded = STANDARD.encode(ed25519_keypair.to_bytes());
    
    // Créer le répertoire parent s'il n'existe pas
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(&path, encoded)?;
    println!("Nouvelle identité sauvegardée dans {:?}", path);

    Ok(ed25519_keypair)
}

/// Retourne un Keypair utilisable par libp2p
pub fn load_identity_file() -> Result<Keypair> {
    let path = get_key_path();
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
    let path = get_info_path();
    let json = serde_json::to_string_pretty(identity)
        .context("Failed to serialize identity info")?;
    
    // Créer le répertoire parent s'il n'existe pas
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(&path, json).context("Failed to write identity file")?;
    Ok(())
}
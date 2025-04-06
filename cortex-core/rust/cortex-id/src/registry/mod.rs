use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Informations sur un shard disponible sur un nœud
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    pub shard_id: String,
    pub version: String,
    pub available: bool,
}

/// Entrée de registre pour un nœud
#[derive(Debug, Clone)]
pub struct NodeEntry {
    pub last_seen: Instant,
    pub shards: Vec<ShardInfo>,
    pub vram_free_mb: u32,
}

/// Message de simulation ou de réception PubSub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceMsg {
    pub node_id: String,
    pub shards: Vec<String>,
    pub version: String,
    pub vram_free_mb: u32,
}

/// Registry local contenant les métadonnées du mesh
#[derive(Debug, Default, Clone)]
pub struct Registry {
    pub nodes: HashMap<String, NodeEntry>, // node_id → info
}

impl Registry {
    /// Met à jour le registre depuis un message d’annonce (ex: PubSub)
    pub fn update_from_announce(&mut self, msg: AnnounceMsg) {
        let shards = msg.shards.into_iter().map(|s| ShardInfo {
            shard_id: s,
            version: msg.version.clone(),
            available: true,
        }).collect();

        let entry = NodeEntry {
            last_seen: Instant::now(),
            shards,
            vram_free_mb: msg.vram_free_mb,
        };

        self.nodes.insert(msg.node_id, entry);
    }

    /// Supprime les nœuds inactifs depuis plus de `ttl` secondes
    pub fn purge_stale(&mut self, ttl: Duration) {
        let now = Instant::now();
        self.nodes.retain(|_, entry| now.duration_since(entry.last_seen) < ttl);
    }

    /// Export JSON lisible pour debug ou snapshot
    pub fn snapshot_json(&self) -> String {
        #[derive(Serialize)]
        struct Snapshot<'a> {
            timestamp: u64,
            nodes: &'a HashMap<String, NodeEntryJson>,
        }

        #[derive(Serialize)]
        struct NodeEntryJson {
            shards: Vec<ShardInfo>,
            vram_free_mb: u32,
            last_seen_secs_ago: u64,
        }

        let now = Instant::now();
        let mapped: HashMap<_, _> = self.nodes.iter().map(|(k, v)| {
            let age = now.duration_since(v.last_seen).as_secs();
            let json = NodeEntryJson {
                shards: v.shards.clone(),
                vram_free_mb: v.vram_free_mb,
                last_seen_secs_ago: age,
            };
            (k.clone(), json)
        }).collect();

        let snap = Snapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
            nodes: &mapped,
        };
        

        serde_json::to_string_pretty(&snap).unwrap_or_else(|_| "{}".into())
    }
}

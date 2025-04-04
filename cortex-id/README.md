```markdown
# cortex-id

Générateur d'identité libp2p pour les nœuds Cortex Mesh.

Ce module Rust permet de générer une clé d'identité `ed25519` persistante et un `PeerId` unique, utilisés pour identifier un nœud dans le réseau distribué Cortex.

## Fonctionnalités

- Génération de clé `ed25519`
- Conversion en `libp2p::identity::Keypair`
- Création du `PeerId`
- Encodage base64 de la clé privée
- Persistance dans `~/.cortex/identity.key`

## Compilation avec Docker

1. Build (statiquement pour Linux musl) :

```bash
docker compose build
```

2. Génération de l'identité :

```bash
docker compose run cortex-id
```

Cela crée automatiquement un fichier :

```bash
~/.cortex/identity.key
```

Exemple de contenu :

```json
{
  "peer_id": "12D3KooW...",
  "key_base64": "..."
}
```

## Structure interne

- `src/lib.rs` : bibliothèque réutilisable (génération + sauvegarde)
- `src/main.rs` : exécutable CLI
- `Dockerfile` : build multi-stage (Rust + Alpine)
- `docker-compose.yml` : orchestration simple

## Exemple de réutilisation dans un autre composant

```rust
let file = std::fs::read_to_string("~/.cortex/identity.key")?;
let parsed: Identity = serde_json::from_str(&file)?;
let decoded = base64::decode(&parsed.key_base64)?;

// Recharger un Keypair à partir de la clé sauvegardée
let ed_kp = ed25519::Keypair::from_bytes(&decoded)?;
let kp = Keypair::from(ed_kp);
let peer_id = PeerId::from_public_key(&kp.public());
```

## Pré-requis système

- Docker
- docker-compose
- Linux, macOS ou WSL (testé sous Ubuntu 22.04)

## Utilité dans Cortex

Dans Cortex, chaque nœud doit posséder une identité stable et traçable :

- Pour s’authentifier dans le réseau P2P (libp2p)
- Pour publier ses shards, recevoir des requêtes et vérifier les signatures

## Licence

MIT — développé dans le cadre du projet Hippocamp / Cortex.
```
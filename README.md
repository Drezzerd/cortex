# Cortex

## Description

**Cortex** est une infrastructure décentralisée conçue pour exécuter des modèles de langage de grande taille (LLM) à travers un réseau pair-à-pair.
Chaque nœud du réseau héberge un fragment du modèle (ex. expert, shard, bloc) et coopère avec les autres pour répondre à une requête.

Le système prend en charge différentes architectures (dense, MoE, pipeline) et s'appuie sur une couche réseau P2P chiffrée, sans point central de coordination.

---

## Objectifs

- Répartir l’exécution d’un modèle MoE sur un ensemble hétérogène de machines.
- Assurer la cohérence, la sécurité et la souveraineté des calculs sans point de contrôle central.
- Fournir un socle modulaire, extensible et indépendant de l’infrastructure cloud.

---

## Caractéristiques principales

- **Architecture P2P** (libp2p, QUIC) auto-organisante.
- **Activation top-k** d’experts par token.
- **Communication haute performance** via canaux dédiés (ZeroMQ, gRPC).
- **Redondance et validation croisée** par quorum.
- **Mode dégradé** possible en local (LLM léger).

---

## État du projet

- Prototype fonctionnel (exécution locale + réseau P2P).
- Modules principaux définis : Loader, Scheduler, TokenRouter, Communicator, Registry.
- Intégration d’un modèle MoE (DeepSeek) en cours de test distribué.

---

## Installation
Voici la section **Installation** mise à jour avec précision et sobriété, dans le ton du reste du README :

---

## Installation

Bien vu. On peut donc ajouter cette étape explicite dans le README. Voici la section **Installation** mise à jour, claire et exacte :

---

## Installation

**Pré-requis :**

- Linux avec Docker + Docker Compose installés
- Git
- Accès à un terminal (avec ou sans `sudo` selon votre config)


```bash
git clone <repo> cortex
cd cortex

chmod +x install.sh        # rendre le script exécutable
./install.sh               # ajoutez 'sudo' si nécessaire
```

> Le script :
> - Crée le dossier `~/.cortex/`
> - Génère une identité réseau (si absente)
> - Détecte les ressources matérielles locales (RAM, CPU, GPU)
> - Génère `~/.cortex/config.yaml`
> - Lance le nœud via `docker compose`

Si une erreur de permission apparaît (ex : accès refusé à `~/.cortex` ou Docker non autorisé), relancez simplement :

```bash
sudo ./install.sh
```

---

## Licence

MIT.

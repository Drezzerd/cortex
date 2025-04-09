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

./install.sh

---

## Licence

MIT.

# Cortex

> L'infrastructure distribuée pour l’exécution parallèle de modèles de langage à grande échelle.

---

## Description

**Cortex** est un moteur d’inférence conçu pour exécuter des modèles de langage de grande taille (LLM) en environnement distribué.
Chaque nœud du réseau héberge une portion du modèle (shard, expert, ou bloc logique) et coopère avec d'autres pour effectuer une inférence complète, sans point central de coordination.

Le système est compatible avec différentes architectures (MoE, dense, pipeline-parallel) et repose sur une couche réseau P2P sécurisée, à faible latence.

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

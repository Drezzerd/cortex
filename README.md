# Cortex

> L'infrastructure distribuée pour l’exécution parallèle de modèles de langage à grande échelle.

---

## Description

**Cortex** est un moteur d’inférence décentralisé permettant l’exécution collective de modèles de langage massifs.  
Chaque instance (appelée *nœud*) héberge un ou plusieurs fragments spécialisés du modèle (experts) et communique avec d’autres nœuds via un réseau pair-à-pair chiffré.

Le système est conçu pour les architectures de type **Mixture of Experts (MoE)**, avec routage dynamique, activation top-k, quorum de validation, et tolérance aux pannes.

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

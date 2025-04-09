# Cortex

## Description

**Cortex** est une infrastructure décentralisée conçue pour exécuter des modèles de langage de grande taille (LLM) à travers un réseau pair-à-pair.
Chaque nœud du réseau héberge un fragment du modèle (ex. expert, shard, bloc) et coopère avec les autres pour répondre à une requête.

Le système prend en charge différentes architectures (dense, MoE, pipeline) et s'appuie sur une couche réseau P2P chiffrée, sans point central de coordination.

---

## Objectifs

Cortex a pour but de permettre l’exécution distribuée de modèles de langage de grande taille (LLM) sur un ensemble de machines autonomes, sans infrastructure centralisée. Les objectifs sont :

### 1. **Scalabilité horizontale**

Permettre à des machines aux capacités limitées de contribuer à l’exécution d’un modèle en réseau. L’infrastructure doit s’adapter dynamiquement au nombre et à la nature des nœuds disponibles (CPU, GPU, RAM).

### 2. **Exécution coopérative de LLM**

Supporter plusieurs types d’architectures (dense, MoE, pipeline) en décomposant un modèle en unités exécutables distribuées (shards, experts, blocs). L’inférence complète est obtenue via coopération inter-nœuds.

### 3. **Décentralisation totale**

Aucune autorité centrale. Le routage, la découverte de pairs, la distribution des poids et la coordination doivent être entièrement distribués (via DHT, PubSub, etc.).

### 4. **Souveraineté et confidentialité**

Les contextes d’inférence restent localisés. Aucun transfert d’historique, de prompt ou de résultat n’est effectué sans consentement. Le chiffrement et la signature sont utilisés à tous les niveaux.

### 5. **Tolérance aux pannes et reconfiguration**

Le système doit détecter la disparition ou l’apparition de nœuds, réassigner les tâches automatiquement, et maintenir un service partiel en cas de défaillance. Reconfiguration à chaud.

---

## Valeurs

Cortex est guidé par des principes fondamentaux, non négociables :

- **Confidentialité par conception**  
  Aucun échange de données n’est effectué sans contrôle explicite. Les contextes utilisateurs restent locaux.

- **Souveraineté computationnelle**  
  Chaque nœud exécute ce qu’il comprend et contrôle. Aucune dépendance à une entité centrale ou cloud propriétaire.

- **Transparence et vérifiabilité**  
  Le fonctionnement du système est ouvert, inspectable, reproductible.

- **Résilience collective**  
  Le réseau doit continuer de fonctionner malgré les déconnexions, les pannes ou les hétérogénéités matérielles.

- **Neutralité d’usage**  
  Cortex ne restreint pas les cas d’usage par design. Ce sont les pairs qui définissent la gouvernance, s’il y en a une.

---

## Caractéristiques principales

- **Architecture P2P** (libp2p, QUIC) auto-organisante.
- **Activation top-k** d’experts par token.
- **Communication haute performance** via canaux dédiés (ZeroMQ, gRPC).
- **Redondance et validation croisée** par quorum.
- **Mode dégradé** possible en local (LLM léger).

---

## État du projet

- Découverte de pairs fonctionnelle via libp2p (mDNS et/ou DHT)
- Génération d'identité persistente par nœud
- Initialisation automatisée via script (install.sh)
- Communication inter-nœuds (mesh) en cours d’expérimentation
- Aucune exécution de modèle encore implémentée

---

## Installation

**Pré-requis :**

- Docker + Docker Compose installés
- Git


```bash
git clone https://github.com/Drezzerd/cortex.git
cd cortex

chmod +x install.sh
./install.sh 
```

> Le script :
> - Crée le dossier `~/.cortex/`
> - Génère une identité réseau (si absente)
> - Détecte les ressources matérielles locales (RAM, CPU, GPU)
> - Génère `~/.cortex/config.yaml`
> - Lance le nœud via `docker compose`

Si une erreur de permission apparaît (ex : accès refusé à `~/.cortex` ou Docker non autorisé), relancez :

```bash
sudo ./install.sh
```

---

## Licence

MIT

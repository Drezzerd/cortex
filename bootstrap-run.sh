#!/bin/bash

set -e

# Construire l'image si nécessaire
docker compose build

# Assurez‑vous que le dossier existe et qu’il vous appartient
mkdir -p "$HOME/.cortex"
chmod 700 "$HOME/.cortex"
chown "$(id -u)":"$(id -g)" "$HOME/.cortex"

# Lancer le conteneur bootstrap
docker run -it \
  --name cortex-bootstrap \
  --network host \
  -v "$HOME/.cortex:/home/cortexuser/.cortex" \
  -e RUST_LOG=info,libp2p=debug \
  cortex-id:latest \
  --mode bootstrap


# Sortie conteneur: affiche les adresses d'écoute
# Copier l'adresse pour l'utiliser avec les light nodes
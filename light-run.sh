#!/bin/bash

set -e

# Vérifier si l'adresse bootstrap est fournie
if [ -z "$1" ]; then
    echo "Usage: $0 <adresse-bootstrap>"
    echo "Exemple: $0 /ip4/192.168.1.10/udp/50123/quic-v1/p2p/12D3KooWxxxxxx"
    exit 1
fi

BOOTSTRAP_PEER="$1"

# Construire l'image si nécessaire
docker compose build

# Lancer le conteneur light node
docker run -it \
    --name cortex-light \
    --network host \
    -v "$HOME/.cortex:/home/cortexuser/.cortex" \
    -e RUST_LOG=info,libp2p=debug \
    -e CORTEX_BOOTSTRAP_PEER="$BOOTSTRAP_PEER" \
    cortex-id:latest \
    cortex-id --mode light --bootstrap-peer "$BOOTSTRAP_PEER"
#!/bin/bash

set -e

IMAGE="cortex-id"
NAME="cortex-id"

# Supprime l'ancien conteneur si présent
if docker ps -a --format '{{.Names}}' | grep -Eq "^${NAME}\$"; then
    echo "⚠️  Conteneur $NAME déjà existant, suppression..."
    docker rm -f "$NAME"
fi

BOOTSTRAP_PEER=${CORTEX_BOOTSTRAP_PEER:-""}
BOOTSTRAP_OPT=""
if [ -n "$BOOTSTRAP_PEER" ]; then
    BOOTSTRAP_OPT="-e CORTEX_BOOTSTRAP_PEER=$BOOTSTRAP_PEER"
    echo "🌐 Bootstrapping avec le peer: $BOOTSTRAP_PEER"
fi

docker run -it \
    --name "$NAME" \
    --network host \
    -v "$HOME/.cortex:/root/.cortex" \
    $BOOTSTRAP_OPT \
    "$IMAGE"

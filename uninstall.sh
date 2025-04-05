#!/bin/bash
set -e

echo "==> Désinstallation complète du nœud Cortex..."

CORTEX_DIR="$HOME/.cortex"

if [ -d "$CORTEX_DIR" ]; then
    echo "Suppression de $CORTEX_DIR"
    rm -rf "$CORTEX_DIR"
else
    echo "Aucun dossier $CORTEX_DIR à supprimer."
fi

echo "Arrêt et suppression des conteneurs Docker"
docker compose down --volumes --remove-orphans || true

IMAGE_NAME="cortex-id:latest"
IMAGE_EXISTS=$(docker images -q "$IMAGE_NAME")

if [ -n "$IMAGE_EXISTS" ]; then
    echo "Suppression de l'image Docker : $IMAGE_NAME"
    docker image rm "$IMAGE_NAME" --force
else
    echo "Aucune image nommée $IMAGE_NAME à supprimer."
fi

echo "Suppression des images <none> non utilisées..."
docker image prune -f

echo "Désinstallation terminée."

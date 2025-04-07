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

echo "Arrêt et suppression des conteneurs Docker via docker-compose"
docker compose down --volumes --remove-orphans || true

echo "Recherche et suppression des conteneurs nommés cortex-*"
for container in $(docker ps -a --format '{{.Names}}' | grep '^cortex-'); do
    echo "Suppression du conteneur $container"
    docker rm -f "$container"
done

IMAGE_NAME="cortex-id:latest"
IMAGE_EXISTS=$(docker images -q "$IMAGE_NAME")

if [ -n "$IMAGE_EXISTS" ]; then
    echo "Suppression de l'image Docker : $IMAGE_NAME"
    docker image rm "$IMAGE_NAME" --force
else
    echo "Aucune image nommée $IMAGE_NAME à supprimer."
fi

echo "Suppression des images <none> et conteneurs associés..."
for image_id in $(docker images -f "dangling=true" -q); do
  for container_id in $(docker ps -a -q --filter ancestor="$image_id"); do
    echo "Conteneur orphelin : $container_id"
    docker rm -f "$container_id"
  done
  echo "Suppression de l’image orpheline $image_id"
  docker rmi -f "$image_id"
done

echo "✅ Désinstallation terminée."

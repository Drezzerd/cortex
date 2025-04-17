#!/usr/bin/env bash
set -euo pipefail

echo "==> Désinstallation complète du nœud Cortex..."

CORTEX_DIR="$HOME/.cortex"

# Nettoyage du dossier local ~/.cortex
if [ -d "$CORTEX_DIR" ]; then
    echo "Suppression de $CORTEX_DIR"
    rm -rf "$CORTEX_DIR"
else
    echo "Aucun dossier $CORTEX_DIR à supprimer."
fi

# Arrêt et suppression des conteneurs Docker via docker-compose
echo "Arrêt et suppression des conteneurs Docker via docker-compose"
docker compose down --volumes --remove-orphans || true

# Suppression des conteneurs nommés cortex-* (anciennes versions)
echo "Recherche et suppression des conteneurs nommés cortex-*"
for container in $(docker ps -a --format '{{.Names}}' | grep '^cortex-'); do
    echo "Suppression du conteneur $container"
    docker rm -f "$container" || true
done

# Suppression de l'image Docker cortex-id:latest
IMAGE_NAME="cortex-id:latest"
echo "Suppression de l'image Docker : $IMAGE_NAME"
docker image rm "$IMAGE_NAME" --force || echo "Aucune image $IMAGE_NAME à supprimer."

# Nettoyage des images <none> et des conteneurs orphelins
echo "Nettoyage des images <none> et des conteneurs associés..."
for image_id in $(docker images -f "dangling=true" -q); do
  for container_id in $(docker ps -a -q --filter ancestor="$image_id"); do
    echo "Suppression du conteneur orphelin : $container_id"
    docker rm -f "$container_id" || true
  done
  echo "Suppression de l’image orpheline $image_id"
  docker rmi -f "$image_id" || true
done

echo "✅ Désinstallation terminée."

#!/bin/bash

API_URL="http://localhost:8080/announce"

# Générer un ID de nœud fictif
NODE_ID="test-node-$(date +%s)"

# Créer le payload JSON pour l'annonce
JSON_PAYLOAD=$(cat <<END
{
  "node_id": "$NODE_ID",
  "shards": ["stable-diffusion", "mistral"],
  "version": "1.0.0",
  "vram_free_mb": 8192
}
END
)

# Envoyer la requête POST à l'API
echo "Annonce du nœud $NODE_ID..."
curl -X POST $API_URL \
  -H "Content-Type: application/json" \
  -d "$JSON_PAYLOAD"

# Vérifier le registry
echo -e "\n\nVérification du registry..."
curl http://localhost:8080/registry

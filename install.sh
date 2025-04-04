#!/bin/bash
set -e

# Récupère l'utilisateur réel
REAL_USER=${SUDO_USER:-$USER}
USER_HOME=$(getent passwd "$REAL_USER" | cut -d: -f6)
CORTEX_DIR="$USER_HOME/.cortex"
IDENTITY_FILE="$CORTEX_DIR/identity.key"
CONFIG_FILE="$CORTEX_DIR/config.yaml"

# Vérifie si on a les droits d'écriture dans ~/.cortex
mkdir -p "$CORTEX_DIR" 2>/dev/null || {
  echo "Impossible d'accéder à $CORTEX_DIR"
  echo "Relance ce script avec sudo :"
  echo "sudo $0"
  exit 1
}

echo "==> Initialisation du nœud Cortex..."

# Génère l'identité uniquement si absente
if [ ! -f "$IDENTITY_FILE" ]; then
    echo "==> Génération d'une identité (rebuild sans cache)..."
    
    # Rebuild sans cache pour s'assurer que le binaire est à jour
    docker compose build --no-cache cortex-id

    # Exécute le conteneur avec le volume monté
    docker compose run --rm \
      -v "$CORTEX_DIR:/root/.cortex" \
      cortex-id
else
    echo "==> Identité déjà présente : $IDENTITY_FILE"
fi

# Détection hardware
RAM_GB=$(free -g | awk '/Mem:/ { print $2 }')
CPU_CORES=$(nproc)
HAS_GPU=$(lspci | grep -i nvidia &> /dev/null && echo true || echo false)
HOSTNAME=$(hostname)

# Génération du fichier config.yaml
echo "==> Génération de la configuration : $CONFIG_FILE"
cat > "$CONFIG_FILE" <<EOF
identity: $IDENTITY_FILE
hostname: $HOSTNAME

hardware:
  ram_gb: $RAM_GB
  cpu_cores: $CPU_CORES
  gpu: $HAS_GPU

roles:
  shard_executor: $HAS_GPU
  router: true
  monitor: true

mesh:
  pubsub_topic: cortex-v1
EOF

echo "==> Lancement du node Cortex via docker-compose"
docker compose up -d

echo "Installation terminée. Le nœud est prêt."

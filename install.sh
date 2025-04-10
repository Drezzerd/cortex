#!/bin/bash
set -e

REAL_USER=${SUDO_USER:-$USER}
USER_HOME=$(eval echo "~$REAL_USER")
CORTEX_DIR="$USER_HOME/.cortex"
IDENTITY_FILE="$CORTEX_DIR/identity.key"
CONFIG_FILE="$CORTEX_DIR/config.yaml"

# Vérifie que Docker est installé
if ! command -v docker &>/dev/null; then
  echo "❌ Docker n'est pas installé. Installe Docker puis réessaie."
  exit 1
fi

# Crée le dossier ~/.cortex si possible
if [ ! -d "$CORTEX_DIR" ]; then
  echo "==> Création de $CORTEX_DIR"
  mkdir -p "$CORTEX_DIR"
  chown "$REAL_USER" "$CORTEX_DIR"
fi

echo "==> Initialisation du nœud Cortex..."

# Génération d'identité si manquante
if [ ! -f "$IDENTITY_FILE" ]; then
    echo "==> Génération d'une identité..."
    docker compose build cortex-id
    docker compose run --rm -v "$CORTEX_DIR":/root/.cortex cortex-id
else
    echo "==> Identité déjà présente : $IDENTITY_FILE"
fi

# Récupération des specs machine (macOS + Linux)
if command -v free &>/dev/null; then
  RAM_GB=$(free -g | awk '/Mem:/ { print $2 }')
else
  RAM_GB=$(($(sysctl -n hw.memsize) / 1073741824))  # macOS fallback
fi

if command -v nproc &>/dev/null; then
  CPU_CORES=$(nproc)
else
  CPU_CORES=$(sysctl -n hw.ncpu)  # macOS fallback
fi

if command -v lspci &>/dev/null; then
  HAS_GPU=$(lspci | grep -i nvidia &>/dev/null && echo true || echo false)
else
  HAS_GPU=false  # macOS n’a pas lspci ou GPU Nvidia en général
fi

HOSTNAME=$(hostname)

# Création du fichier de configuration
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

# Lancement du conteneur
echo "==> Lancement du node Cortex via docker-compose"
docker compose up -d

echo "✅ Installation terminée. Le nœud est prêt."

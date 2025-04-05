#!/bin/bash
set -e

echo "🚀 Démarrage du nœud Cortex..."

# Vérifie si Rust est installé
if ! command -v cargo &> /dev/null; then
  echo "❌ Rust non détecté. Installe Rust : https://rustup.rs"
  exit 1
fi

# Vérifie si Docker est actif
if ! docker info &> /dev/null; then
  echo "🔧 Docker semble arrêté, tentative de démarrage..."
  sudo systemctl start docker
fi

# Crée ~/.cortex si absent
CORTEX_HOME="${HOME}/.cortex"
mkdir -p "$CORTEX_HOME"

# Affiche l'identité (si déjà générée)
if [ -f "$CORTEX_HOME/identity.key" ]; then
  echo "🔐 Identité déjà présente : $CORTEX_HOME/identity.key"
else
  echo "⚠️ Identité non trouvée. Lance d'abord ./install.sh"
  exit 1
fi

# Positionne dans le dossier Rust cortex-id
cd "$(dirname "$0")/rust/cortex-id"

# Compile et lance le nœud libp2p
cargo run --bin cortex-node

use cortex_id::discovery::run_discovery;
use cortex_id::identity::load_or_generate_identity;
use cortex_id::registry::Registry;
use libp2p::identity::Keypair;
use clap::Parser;
use anyhow::Result;

/// Cortex Node CLI
#[derive(Parser, Debug)]
#[command(name = "cortex-id")]
#[command(about = "Lance un noeud Cortex avec options", long_about = None)]
struct Cli {
    /// Affiche une snapshot unique du registry puis quitte
    #[arg(long)]
    snapshot: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Chargement ou génération de l'identité
    let identity: Keypair = load_or_generate_identity()?.into();

    // Mode simple : on lance la découverte
    if cli.snapshot {
        // Démarrage silencieux, capture snapshot et exit
        let reg = Registry::default();
        println!("📦 Snapshot du Registry:");
        println!("{}", reg.snapshot_json());
        return Ok(());
    } else {
        // Mode normal : on lance le nœud Cortex complet
        run_discovery(identity).await
    }
}
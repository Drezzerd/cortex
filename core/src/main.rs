use cortex_id::discovery::{run_bootstrap_node, run_light_node};
use cortex_id::identity::load_or_generate_identity;
use clap::Parser;
use std::env;
use anyhow::Result;

/// Cortex Node CLI
#[derive(Parser, Debug)]
#[command(name = "cortex-id")]
#[command(about = "Lance un noeud Cortex avec options", long_about = None)]
struct Cli {
    /// Mode de fonctionnement: bootstrap ou light
    #[arg(long, default_value = "light")]
    mode: String,
    
    /// Adresse bootstrap à utiliser (format multiaddr)
    #[arg(long)]
    bootstrap_peer: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialisation des logs pour le debugging
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .filter_module("libp2p", log::LevelFilter::Debug)
        .init();
    
    let cli = Cli::parse();
    
    // Priorité: argument CLI, puis variable d'environnement, puis défaut
    let mode = cli.mode.clone();
    
    // Pour l'adresse bootstrap, si fournie en CLI, on l'utilise
    if let Some(bootstrap) = cli.bootstrap_peer {
        env::set_var("CORTEX_BOOTSTRAP_PEER", bootstrap);
    }
    
    // Chargement ou génération de l'identité
    println!("🔑 Chargement/génération de l'identité...");
    let keypair = load_or_generate_identity()?.into();
    
    println!("🚀 Démarrage du nœud en mode: {}", mode);
    
    // Choix du mode de fonctionnement
    match mode.as_str() {
        "bootstrap" => run_bootstrap_node(keypair).await,
        "light" => run_light_node(keypair).await,
        _ => {
            println!("Mode inconnu: {}, utilisation du mode light par défaut", mode);
            run_light_node(keypair).await
        }
    }
}
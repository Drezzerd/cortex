use cortex_id::discovery::run_discovery;
use cortex_id::identity::load_or_generate_identity;

use std::panic;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node_name = std::env::var("NODE_NAME").unwrap_or_else(|_| "unknown".into());
    println!("🚀 Starting node: {}", node_name);

    let keypair = load_or_generate_identity()?;
    println!("✅ Identity loaded");

    run_discovery(keypair.into()).await
}


async fn run() -> anyhow::Result<()> {
    let keypair = load_or_generate_identity()?;
    run_discovery(keypair.into()).await
}

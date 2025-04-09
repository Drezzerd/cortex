use cortex_id::discovery::run_bootstrap_node;
use cortex_id::identity::load_or_generate_identity;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let keypair = load_or_generate_identity()?.into();
    run_bootstrap_node(keypair).await?;
    Ok(())
}
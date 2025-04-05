use cortex_id::discovery::run_discovery;
use cortex_id::identity::load_or_generate_identity;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let keypair = load_or_generate_identity()?;
    run_discovery(keypair.into()).await
}

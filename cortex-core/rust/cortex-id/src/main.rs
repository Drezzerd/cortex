use cortex_id::identity::{generate_identity, save_identity_file};

fn main() -> anyhow::Result<()> {
    let identity = generate_identity()?;
    save_identity_file(&identity)?;
    println!("Identité générée : {}", identity.peer_id);
    Ok(())
}
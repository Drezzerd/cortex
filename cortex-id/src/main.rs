use cortex_id::{generate_identity, save_identity_file};

fn main() {
    let id = generate_identity();
    println!("Peer ID : {}", id.peer_id);

    match save_identity_file(&id) {
        Ok(_) => println!("Identity saved to ~/.cortex/identity.key"),
        Err(e) => eprintln!("Failed to save identity: {}", e),
    }
}

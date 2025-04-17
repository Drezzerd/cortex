// src/api_interface/mod.rs
use warp::Filter;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use warp::reject::Reject;
use anyhow::Error as AnyhowError;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::convert::Infallible;

use crate::communicator::{CommunicatorMessage, SharedCommunicator};
use crate::registry::Registry;

#[derive(Debug)]
struct ApiError(AnyhowError);

impl Reject for ApiError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRequest {
    pub query: String,
    // Ajouter d'autres champs au besoin
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub response: String,
    // Champ pour les erreurs éventuelles ou les métadonnées
}

// Filtre qui injecte le registry partagé
fn with_registry(registry: Arc<Mutex<Registry>>)
    -> impl Filter<Extract = (Arc<Mutex<Registry>>,), Error = Infallible> + Clone
{
    warp::any().map(move || registry.clone())
}

/// Endpoint pour obtenir le snapshot du registry
async fn handle_registry(registry: Arc<Mutex<Registry>>) -> Result<impl warp::Reply, Infallible> {
    let reg = registry.lock().await;
    let snapshot = reg.snapshot_json();
    // Retourner directement le snapshot comme réponse JSON brute
    Ok(warp::reply::with_header(
        snapshot,
        "content-type",
        "application/json",
    ))
}

/// Lance l'API et attend des requêtes sur l'endpoint /send
pub async fn run_api_server(
    port: u16, 
    communicator: SharedCommunicator,
    registry: Arc<Mutex<Registry>>
) {
    // La route "send" accepte des requêtes POST avec un JSON correspondant à ApiRequest
    let send_route = warp::path("send")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_communicator(communicator))
        .and_then(handle_send);

    // Ajout de la route pour consulter le registry
    let registry_route = warp::path("registry")
        .and(warp::get())
        .and(with_registry(registry))
        .and_then(handle_registry);

    // Combinaison des routes
    let routes = send_route.or(registry_route);

    println!("Lancement du serveur API sur le port {}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

/// Fonction pour injecter le SharedCommunicator dans la route
fn with_communicator(communicator: SharedCommunicator)
    -> impl Filter<Extract = (SharedCommunicator,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || communicator.clone())
}

/// Fonction de gestion de la requête API
async fn handle_send(req: ApiRequest, communicator: SharedCommunicator) 
    -> Result<impl warp::Reply, warp::Rejection> 
{
    // Création d'un CommunicatorMessage avec les infos nécessaires
    let message = CommunicatorMessage {
        sender: "API_Interface".to_string(),
        payload: req.query,
        timestamp: Utc::now().timestamp_millis() as u64,
    };

    // On verrouille le communicator pour envoyer le message
    let mut comm = communicator.lock().await;
    match comm.send_message(&message) {
        Ok(_) => Ok(warp::reply::json(&ApiResponse {
            response: "Message envoyé avec succès".to_string(),
        })),
        Err(e) => {
            eprintln!("Erreur lors de l'envoi du message: {:?}", e);
            Err(warp::reject::custom(ApiError(e)))
        }
    }
}
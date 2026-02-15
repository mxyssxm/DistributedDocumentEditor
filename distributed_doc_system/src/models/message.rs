use actix::prelude::*;
use serde::{Deserialize, Serialize};
use crate::models::document::Document;

// --- DTOs pour les requêtes HTTP (Navigateur -> Serveur) ---
#[derive(Deserialize)]
pub struct CreateReq {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateReq {
    // Le nouveau texte du document.
    pub content: String,
    // détecter les conflits si quelqu'un d'autre a modifié le doc entre-temps
    pub version: usize,
}

#[derive(Message)]
#[rtype(result = "Vec<Document>")]
// Un message "signal" sans paramètre pour récupérer tout les Docs.
pub struct GetDocs;

#[derive(Message)]
#[rtype(result = "Option<Document>")]
// Demande de lecture d'un document spécifique.
pub struct GetDoc {
    // L'identifiant cible.
    pub doc_id: String,
}

#[derive(Message)]
#[rtype(result = "Document")]
// ordre de création envoyé au Repository
pub struct CreateDoc {
    // Le nom choisi par l'utilisateur.
    pub name: String,
}

#[derive(Message)]
#[rtype(result = "Result<Document, String>")]
// Ordre de modification, correspond à l'action Update
pub struct UpdateDoc {
    // L'ID du document à modifier
    pub doc_id: String,
    // Le nouveau texte.
    pub content: String,
    // La version de base pour la validation
    pub version: usize,
}

// Message venant du réseau (Zenoh -> Base de données)
#[derive(Message)]
#[rtype(result = "()")]
// Matérialise les messages venant du "Pub/Sub
pub struct NetworkUpdate {
    // (Key Expression) pour savoir quel document est impacté par le réseau.
    pub key: String,
    pub payload: String,
}
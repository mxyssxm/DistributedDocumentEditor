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
    pub content: String,
    pub version: usize,
}

// --- Messages internes pour l'acteur (API -> Base de données) ---
#[derive(Message)]
#[rtype(result = "Vec<Document>")]
pub struct GetDocs;

#[derive(Message)]
#[rtype(result = "Option<Document>")]
pub struct GetDoc {
    pub doc_id: String,
}

#[derive(Message)]
#[rtype(result = "Document")]
pub struct CreateDoc {
    pub name: String,
}

#[derive(Message)]
#[rtype(result = "Result<Document, String>")]
pub struct UpdateDoc {
    pub doc_id: String,
    pub content: String,
    pub version: usize,
}

// Message venant de Zenoh (Réseau -> Base de données)
#[derive(Message)]
#[rtype(result = "()")]
pub struct ZenohUpdate {
    pub key: String,
    pub payload: String,
}
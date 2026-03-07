use actix::prelude::*;
use serde::Deserialize;
use crate::models::document::Document;

#[derive(Deserialize)]
pub struct CreateRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub content: String,
    pub version: usize,
}

#[derive(Message)]
#[rtype(result = "Vec<Document>")]
pub struct GetAllDocuments;

#[derive(Message)]
#[rtype(result = "Option<Document>")]
pub struct GetSingleDocument {
    pub doc_id: String,
}

#[derive(Message)]
#[rtype(result = "Document")]
pub struct CreateDoc {
    pub name: String,
    pub client_id: String,
}

#[derive(Message)]
#[rtype(result = "Result<Document, String>")]
pub struct UpdateDoc {
    pub doc_id: String,
    pub content: String,
    pub version: usize,
    pub client_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NetworkUpdate {
    pub key: String,
    pub payload: String,
}
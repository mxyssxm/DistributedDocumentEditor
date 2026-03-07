use actix_web::{web, HttpResponse, Responder};
use actix::Addr;
use crate::services::doc_server::DocServer;
use crate::models::message::*;

pub async fn get_docs(doc_server: web::Data<Addr<DocServer>>) -> impl Responder {
    match doc_server.send(GetAllDocuments).await {
        Ok(documents) => HttpResponse::Ok().json(documents),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_doc(path: web::Path<String>, doc_server: web::Data<Addr<DocServer>>) -> impl Responder {
    match doc_server.send(GetSingleDocument { doc_id: path.into_inner() }).await {
        Ok(Some(document)) => HttpResponse::Ok().json(document),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn create_doc(request: web::Json<CreateRequest>, doc_server: web::Data<Addr<DocServer>>) -> impl Responder {
    match doc_server.send(CreateDoc { name: request.name.clone(), client_id: "Interface_Web".to_string() }).await {
        Ok(document) => HttpResponse::Ok().json(document),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_doc(path: web::Path<String>, request: web::Json<UpdateRequest>, doc_server: web::Data<Addr<DocServer>>) -> impl Responder {
    match doc_server.send(UpdateDoc { 
        doc_id: path.into_inner(), 
        content: request.content.clone(), 
        version: request.version,
        client_id: "Interface_Web".to_string() 
    }).await {
        Ok(Ok(document)) => HttpResponse::Ok().json(document),
        Ok(Err(error_message)) => HttpResponse::Conflict().body(error_message),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
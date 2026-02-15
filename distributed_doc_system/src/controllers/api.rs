
// - web : Permet d'extraire les données des requêtes Chemin de l'URL
// - HttpResponse : Permet de construire les réponses HTTP  (404 Not Found, etc.).
// - Responder : Le Type de retour standard d'une route Actix.

use actix_web::{web, HttpResponse, Responder};
use actix::Addr;
use crate::services::doc_server::DocServer;
use crate::models::message::*;

// Fonction asynchrone qui gère la route. 
// Paramètre : 'srv' est l'adresse de notre Acteur
pub async fn get_docs(srv: web::Data<Addr<DocServer>>) -> impl Responder {
    // On envoie le message 'GetDocs' à l'acteur. '.await' met la requête HTTP en pause
    match srv.send(GetDocs).await {
        // Si l'acteur a répondu correctement, on renvoie un HTTP 200 (Ok) 
        // et on transforme la liste des documents en format JSON pour le client.
        Ok(docs) => HttpResponse::Ok().json(docs),
        // on renvoie une erreur HTTP
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

//Extrait dynamiquement l'ID du document depuis l'URL
pub async fn get_doc(path: web::Path<String>, srv: web::Data<Addr<DocServer>>) -> impl Responder {
    // On envoie le message 'GetDoc' en lui passant l'ID extrait
    match srv.send(GetDoc { doc_id: path.into_inner() }).await {
        // L'acteur a répondu "Oui, je l'ai trouvé" (Some). 
        // On renvoie HTTP 200 (Ok)
        Ok(Some(doc)) => HttpResponse::Ok().json(doc),
        // L'acteur a répondu "Non, l'ID n'existe pas" (None).
        // On renvoie proprement un HTTP 404
        Ok(None) => HttpResponse::NotFound().finish(),
        // Problème de communication avec l'acteur = HTTP 500
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Paramètre 'req' : Actix-Web va lire le Body de la requête HTTP, vérifier que c'est du JSON,
pub async fn create_doc(req: web::Json<CreateReq>, srv: web::Data<Addr<DocServer>>) -> impl Responder {
    // On demande à l'acteur de créer le document avec le nom fourni (req.name)
    match srv.send(CreateDoc { name: req.name.clone() }).await {
        // L'acteur nous renvoie le nouveau document complet (avec son nouvel ID et version 0).
        // On renvoie HTTP 200 (Ok)
        Ok(doc) => HttpResponse::Ok().json(doc),
        // Problème interne = HTTP 500.
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Paramètre 'path' : L'ID du document ciblé dans l'URL.
// Paramètre 'req' : Le nouveau contenu
pub async fn update_doc(path: web::Path<String>, req: web::Json<UpdateReq>, srv: web::Data<Addr<DocServer>>) -> impl Responder {
    // On prépare le message 'UpdateDoc' pour l'acteur avec toutes les infos.
    match srv.send(UpdateDoc { 
        doc_id: path.into_inner(), 
        content: req.content.clone(), 
        version: req.version 
    }).await {
        // Double 'Ok' : La communication Actix a marché (Ok 1) ET l'acteur a réussi la mise à jour (Ok 2).
        // On renvoie le document mis à jour en JSON (HTTP 200)
        Ok(Ok(doc)) => HttpResponse::Ok().json(doc),
        // L'acteur a rejeté la modification (ex: conflit de version) !
        Ok(Err(e)) => HttpResponse::Conflict().body(e),
        // Crash du système de messagerie de l'acteur = HTTP 500.
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
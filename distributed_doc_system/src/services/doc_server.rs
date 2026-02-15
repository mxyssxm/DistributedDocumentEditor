use actix::prelude::*;
use std::sync::Arc;
use zenoh::Session;
use crate::models::document::Repository;

pub struct DocServer {
    // Le stockage des documents. 
    // pub(crate) signifie que les autres fichiers de notre projet peuvent y accéder,
    pub(crate) repo: Repository,
    // La connexion au "Pub/Sub" 
    // L'utilisation de Arc<Session> garantit la continuation et gerer les conflits.
    pub(crate) zenoh_session: Arc<Session>,
}

impl DocServer {
    // Fonction appelée une seule fois au démarrage de l'application
    pub fn new(zenoh_session: Arc<Session>) -> Self {
        Self {
            // Initialise une base de données vide
            repo: Repository::new(),
            network_session,
        }
    }
}

impl Actor for DocServer {
    type Context = Context<Self>;
}


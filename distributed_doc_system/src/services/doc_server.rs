use actix::prelude::*;
use std::collections::HashMap;
use serde_json::to_string;
use std::sync::Arc;
use zenoh::Session;

use crate::models::message::{ServerMessage, WsMessage};
use crate::models::document::Repository;

pub struct DocServer { // Définition de la structure principale du Serveur.
    
    pub(crate) sessions: HashMap<usize, Recipient<WsMessage>>, // Liste des connexions actives : associe un ID Client
    pub(crate) repo: Repository,
    pub(crate) zenoh_session: Arc<Session>,
}

impl DocServer {
    // Le Constructeur : Initialise le serveur au démarrage de l'application.
    pub fn new(zenoh_session: Arc<Session>) -> Self {
        Self {
            // Au démarrage, il n'y a aucun utilisateur connecté.
            sessions: HashMap::new(),
            // Au démarrage, on crée un stockage de documents vide.
            repo: Repository::new(),
            // On lui passe la connexion au réseau Zenoh.
            zenoh_session,
        }
    }
    //  Diffusion  à tout le monde;"Notify"
    pub(crate) fn broadcast(&self, message: &ServerMessage) {
        // Transforme l'objet Rust en texte JSON
        let msg_str = to_string(message).unwrap();
        // Boucle sur tous les utilisateurs actuellement connectés
        for addr in self.sessions.values() {
            let _ = addr.do_send(WsMessage(msg_str.clone()));
        }
    }
    //  Envoyer à un seul utilisateur.
    pub(crate) fn send_to(&self, client_id: usize, message: &ServerMessage) {
        // Cherche le  WebSocket  de cet utilisateur grâce à son ID.
        if let Some(addr) = self.sessions.get(&client_id) {
            // Si l'utilisateur est bien là, on transforme le message en JSON.
            let msg_str = to_string(message).unwrap();
            // On lui envoie le message à lui uniquement.
            let _ = addr.do_send(WsMessage(msg_str));
        }
    }
}
//permet de recevoir des messages  de manière asynchrone sans jamais bloquer l'application.
impl Actor for DocServer {
    type Context = Context<Self>;
}



use actix::prelude::*;
use serde::{Deserialize, Serialize};

// les Messages envoyés par le Client -> Serveur
pub enum ClientMessage {
    Create { name : String},
    Update { id : String, content : String, version : usize },
    Join { id : String},

}

// les Messages envoyés par le Serveur -> Client 
pub enum ServerMessage {
    DocCreated { id : String, name : String},
    DocUpdate { id : String, content : String, version : String},
    Error { message : String}, 
    List { docs: Vec<(String, String)> },

}

pub struct WsMessage(pub String);

// Connexion de  l'utilisateur
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub id: usize,
}

// Déconnexion
pub struct Disconnect {
    pub id: usize,
}

pub struct ClientCommand{
    pub id : usize,
    pub cmd : ClientMessage,
}
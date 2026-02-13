use actix::prelude::*;
use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]  //configure le format JSON pour qu'il soit facile à lire par le frontend

pub enum ClientMessage {   // Définit strictement les actions possibles de l'utilisateur
    Create { name: String },
    Update { doc_id: String, content: String, version: usize }, 
    Join { doc_id: String }, 
}

#[derive(Serialize, Deserialize, Debug, Clone)] // permet la conversion en JSON pour la réponse réseau.
#[serde(tag = "type", content = "payload")]

pub enum ServerMessage {   // SERVEUR -> CLIENT
    DocCreated { doc_id: String, name: String }, 
    DocUpdated { doc_id: String, content: String, version: usize }, 
    Error { message: String },   // si on a un Conflit de version détecté
    List { docs: Vec<(String, String)> }, // Envoie la liste des documents disponibles à la connexion.
}



#[derive(Message)]
#[rtype(result = "()")] // Définit que l'acteur qui reçoit ce message ne renvoie RIEN

pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]    //  Réponse vide attendue

pub struct Connect {        // Événement : Un User ouvre une connexion WebSocket.
    pub addr: Recipient<WsMessage>,     // L'adresse de réponse
    pub client_id: usize, 
}

#[derive(Message)]
#[rtype(result = "()")]

pub struct Disconnect {   // Événement : Un utilisateur ferme son onglet.
    pub client_id: usize,   // Identifie quel utilisateur s'est déconnecté
}

#[derive(Message)]
#[rtype(result = "()")]

pub struct ClientCommand {    
    pub client_id: usize, 
    pub cmd: ClientMessage,    // Contient le Create/Update/Join
}


#[derive(Message, Debug)]
#[rtype(result = "()")]

pub struct ZenohUpdate {       // L'Interface avec le Middleware "Pub/Sub"
    pub key: String,          // (Key Expression) Permet de savoir QUEL document a été modifié.
    pub payload: String,     // C'est le contenu réel  transporté par le middleware
}
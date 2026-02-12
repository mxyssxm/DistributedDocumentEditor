use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub doc_id: String, 
    pub name: String,
    pub content: String,
    pub version: usize,
}

impl Document {    // Constructeur pour Crée un nouveau document vide.
    // Appelé quand le serveur reçoit le message ClientMessage::Create.
    pub fn new(name: String) -> Self {
        Self {
            doc_id: uuid::Uuid::new_v4().to_string(), 
            name,
            content: String::new(),    // Commence vide.
            version: 0,                // Commence à la version 0.
        }
    }

    pub fn update(&mut self, new_content: String, base_version: usize) -> Result<usize, String> {
        // VÉRIFICATION DE CONFLIT 
        // On compare la version que le client PENSE avoir (base_version)
        // avec la version RÉELLE du serveur (self.version).
        
        if base_version != self.version {
            return Err("Conflit : Version obsolète.".to_string()); // On refuse la mise à jour pour ne pas écraser le travail de l'autre.
        }
        // SUCCÈS : Les versions correspondent.
        self.content = new_content;  // 1. On applique le nouveau texte.
        self.version += 1;  // 2. On incrémente la version locale
        Ok(self.version) // On retourne la nouvelle version pour confirmer le succès
    }
}

pub struct Repository {
    // La clé de la Map est le doc_id
    // Valeur (Document)
    pub docs: HashMap<String, Document>,
}

impl Repository {   // Initialise le stockage au démarrage du serveur (dans main.rs)
    pub fn new() -> Self {
        Self { docs: HashMap::new() }  // Démarre avec une liste vide
    }
}
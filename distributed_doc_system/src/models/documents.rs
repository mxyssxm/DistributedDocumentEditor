
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Document {
    pub id : String,
    pub name : String,
    pub content : String,
    pub version : usize,

}

impl Document {
    pub fn new(name: String) -> Self {
        Self{  
            id: uuid::Uuid::new_v4().to_string(),
            name,
            content : String::new(),
            version : 0,  // Gestion de version pour les conflits


        }
    }

    pub fn update(&mut self, new_content: String, base_version: usize) -> Result<usize, String> {
        if base_version != Self.version{
            return Err("Conflit de version : Le document a été modifié par un autre utilisateur.".to_String());

        }
        self.content = new_content;
        self.version += 1;
        OK(self.version)
    }
}

// le stockage  en mémoire
pub struct Repository {
    pub docs: HashMap<String, Document>,
    
}

impl Repository {
    pub fn new() -> Self{
        Self { docs: Hashmap:: new()}

    }
}

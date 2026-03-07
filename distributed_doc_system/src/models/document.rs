use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub doc_id: String, 
    pub name: String,
    pub content: String,
    pub version: usize,
    pub winning_client_id: String,
    pub last_base_version: usize,
}

impl Document {    
    pub fn new(name: String, client_id: String) -> Self {
        Self {
            doc_id: uuid::Uuid::new_v4().to_string(), 
            name,
            content: String::new(),
            version: 0,
            winning_client_id: client_id,
            last_base_version: 0,
        }
    }

    /// Règle métier : Résolution des conflits par ordre lexicographique.
    /// Rejette la mise à jour si la priorité du client est insuffisante.
    pub fn apply_update(&mut self, new_content: String, request_version: usize, client_id: String) -> Result<usize, String> {
        // Cas 1 : Suite logique normale
        if request_version == self.version {
            self.execute_modification(new_content, client_id);
            return Ok(self.version);
        }
        
        // Cas 2 : Conflit concurrent (Deux clients modifient la même base)
        if request_version == self.last_base_version {
            if client_id < self.winning_client_id {
                self.execute_modification(new_content, client_id);
                return Ok(self.version);
            } else {
                return Err("Conflit : Priorité lexicographique insuffisante.".to_string());
            }
        }

        // Cas 3 : Version totalement obsolète
        Err("Conflit : Version obsolète.".to_string())
    }

    
    fn execute_modification(&mut self, new_content: String, client_id: String) {
        self.last_base_version = self.version;
        self.content = new_content;
        self.version += 1;
        self.winning_client_id = client_id;
    }
}

pub struct Repository {
    pub docs: HashMap<String, Document>,
}

impl Repository {
    pub fn new() -> Self {
        Self { docs: HashMap::new() }
    }
}

impl actix::dev::MessageResponse<crate::services::doc_server::DocServer, crate::models::message::CreateDoc> for Document {
    fn handle(self, _ctx: &mut <crate::services::doc_server::DocServer as actix::Actor>::Context, tx: Option<actix::dev::OneshotSender<Self>>) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}
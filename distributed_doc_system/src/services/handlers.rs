use actix::prelude::*;
use crate::services::doc_server::DocServer;
use crate::models::message::*;
use crate::models::document::Document;

impl Handler<GetAllDocuments> for DocServer {
    type Result = Vec<Document>;

    fn handle(&mut self, _: GetAllDocuments, _: &mut Context<Self>) -> Self::Result {
        let mut documents: Vec<Document> = self.repo.docs.values().cloned().collect();
        documents.sort_by(|a, b| a.name.cmp(&b.name));
        documents
    }
}

impl Handler<GetSingleDocument> for DocServer {
    type Result = Option<Document>;

    fn handle(&mut self, msg: GetSingleDocument, _: &mut Context<Self>) -> Self::Result {
        self.repo.docs.get(&msg.doc_id).cloned()
    }
}

impl Handler<CreateDoc> for DocServer {
    type Result = Document;

    fn handle(&mut self, msg: CreateDoc, _: &mut Context<Self>) -> Self::Result {
        let document = Document::new(msg.name, msg.client_id);
        self.repo.docs.insert(document.doc_id.clone(), document.clone());
        
        let topic = format!("docs/{}", document.doc_id);
        let payload = serde_json::to_string(&document).unwrap();
        let net_session = self.network_session.clone(); 
        
        actix::spawn(async move {
            let _ = net_session.put(topic, payload).await;
        });

        document
    }
}

impl Handler<UpdateDoc> for DocServer {
    type Result = Result<Document, String>;
    
    fn handle(&mut self, msg: UpdateDoc, _: &mut Context<Self>) -> Self::Result {
        let update_result = if let Some(document) = self.repo.docs.get_mut(&msg.doc_id) {
            match document.apply_update(msg.content.clone(), msg.version, msg.client_id) {
                Ok(_) => Ok(document.clone()),
                Err(e) => Err(e),
            }
        } else {
            return Err("Document introuvable".to_string());
        };

        if let Ok(updated_doc) = &update_result {
            // Sauvegarde de l'historique pour l'UI
            let history_name = format!("📜 {}.{}", updated_doc.name, updated_doc.version);
            let mut history_doc = updated_doc.clone();
            history_doc.name = history_name;
            self.repo.docs.insert(uuid::Uuid::new_v4().to_string(), history_doc);

            let topic = format!("docs/{}", msg.doc_id);
            let payload = serde_json::to_string(updated_doc).unwrap();
            let net_session = self.network_session.clone(); 
            
            actix::spawn(async move {
                let _ = net_session.put(topic, payload).await;
            });
        }
        
        update_result
    }
}

impl Handler<NetworkUpdate> for DocServer {
    type Result = ();

    fn handle(&mut self, msg: NetworkUpdate, _: &mut Context<Self>) {
        if let Ok(remote_doc) = serde_json::from_str::<Document>(&msg.payload) {
            self.repo.docs.insert(remote_doc.doc_id.clone(), remote_doc.clone());
        }
    }
}



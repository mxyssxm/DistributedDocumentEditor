use actix::prelude::*;
use crate::services::doc_server::DocServer;
use crate::models::message::*;
use crate::models::document::Document;

impl Handler<GetDocs> for DocServer {
    type Result = Vec<Document>;
    fn handle(&mut self, _: GetDocs, _: &mut Context<Self>) -> Self::Result {
        self.repo.docs.values().cloned().collect()
    }
}

impl Handler<GetDoc> for DocServer {
    type Result = Option<Document>;
    fn handle(&mut self, msg: GetDoc, _: &mut Context<Self>) -> Self::Result {
        self.repo.docs.get(&msg.doc_id).cloned()
    }
}

impl Handler<CreateDoc> for DocServer {
    type Result = Document;
    fn handle(&mut self, msg: CreateDoc, _: &mut Context<Self>) -> Self::Result {
        let doc = Document::new(msg.name);
        self.repo.docs.insert(doc.doc_id.clone(), doc.clone());
        
        let topic = format!("docs/{}", doc.doc_id);
        let payload = serde_json::to_string(&doc).unwrap();
        let z_session = self.zenoh_session.clone();
        
        actix::spawn(async move {
            let _ = z_session.put(topic, payload).await;
        });

        doc
    }
}

impl Handler<UpdateDoc> for DocServer {
    type Result = Result<Document, String>;
    fn handle(&mut self, msg: UpdateDoc, _: &mut Context<Self>) -> Self::Result {
        let update_result = if let Some(doc) = self.repo.docs.get_mut(&msg.doc_id) {
            match doc.update(msg.content.clone(), msg.version) {
                Ok(_) => Ok(doc.clone()),
                Err(e) => Err(e),
            }
        } else {
            return Err("Document introuvable".to_string());
        };

        if let Ok(updated_doc) = &update_result {
            let topic = format!("docs/{}", msg.doc_id);
            let payload = serde_json::to_string(updated_doc).unwrap();
            let z_session = self.zenoh_session.clone();
            
            actix::spawn(async move {
                let _ = z_session.put(topic, payload).await;
            });
        }

        update_result
    }
}

impl Handler<ZenohUpdate> for DocServer {
    type Result = ();
    fn handle(&mut self, msg: ZenohUpdate, _: &mut Context<Self>) {
        if let Ok(remote_doc) = serde_json::from_str::<Document>(&msg.payload) {
            println!("📥 Reçu via Zenoh (Topic: {}): {} (v{})", msg.key, remote_doc.name, remote_doc.version);
            self.repo.docs.insert(remote_doc.doc_id.clone(), remote_doc.clone());
        }
    }
}
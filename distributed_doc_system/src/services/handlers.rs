use actix::prelude::*;
use crate::services::doc_server::DocServer;
use crate::models::message::*;
use crate::models::document::Document;

impl Handler<GetDocs> for DocServer {
    // Le type de retour attendu : Une liste (Vector) de Documents.
    type Result = Vec<Document>;

    fn handle(&mut self, _: GetDocs, _: &mut Context<Self>) -> Self::Result {
        // On prend toutes les valeurs de notre HashMap (base de données en mémoire), 
        // on les clone et on en fait une liste.
        self.repo.docs.values().cloned().collect()
    }
}

impl Handler<GetDoc> for DocServer {
    // Le type de retour : Option (soit Some(Document) s'il existe, soit None s'il n'existe pas).
    type Result = Option<Document>;

    fn handle(&mut self, msg: GetDoc, _: &mut Context<Self>) -> Self::Result {
        // On cherche le document par son ID dans la HashMap, et on le clone si on le trouve.
        self.repo.docs.get(&msg.doc_id).cloned()
    }
}

impl Handler<CreateDoc> for DocServer {
    // 🌟 On revient au type simple "Document"
    type Result = Document;

    fn handle(&mut self, msg: CreateDoc, _: &mut Context<Self>) -> Self::Result {
        // 1. On crée le document avec un nouvel UUID généré automatiquement.
        let doc = Document::new(msg.name);
        // 2. On l'insère dans notre base de données locale
        self.repo.docs.insert(doc.doc_id.clone(), doc.clone());
        // 3. On prépare (l'adresse) sur lequel on va publier
        let topic = format!("docs/{}", doc.doc_id);
        // 4. On transforme notre document en texte JSON pour le réseau.
        let payload = serde_json::to_string(&doc).unwrap();
        // 5. On clone le pointeur vers la session réseau Zenoh (grâce à Arc).
        let net_session = self.network_session.clone(); 
        // actix::spawn lance une tâche asynchrone en arrière-plan.
        actix::spawn(async move {
            // pub
            let _ = net_session.put(topic, payload).await;
        });

        // 🌟 On renvoie simplement le document (sans emballage)
        doc
    }
}

impl Handler<UpdateDoc> for DocServer {
    // Soit un Succès avec le Document (Ok), soit une Erreur texte (Err).
    type Result = Result<Document, String>;
    
    fn handle(&mut self, msg: UpdateDoc, _: &mut Context<Self>) -> Self::Result {
        // 1. On cherche le document ciblé, et on demande un accès modifiable (get_mut).
        let update_result = if let Some(doc) = self.repo.docs.get_mut(&msg.doc_id) {
            // 2. On appelle notre fonction d'update (qui vérifie les CONFLITS DE VERSION).
            match doc.update(msg.content.clone(), msg.version) {
                // Succès : La version était bonne, on renvoie le document cloné.
                Ok(_) => Ok(doc.clone()),
                // Échec : Conflit de version détecté, on renvoie l'erreur.
                Err(e) => Err(e),
            }
        } else {
            // Si l'ID n'existe pas du tout.
            return Err("Document introuvable".to_string());
        };

        if let Ok(updated_doc) = &update_result {
            let topic = format!("docs/{}", msg.doc_id);
            // On sérialise la NOUVELLE version du document.
            let payload = serde_json::to_string(updated_doc).unwrap();
            let net_session = self.network_session.clone(); 
            
            actix::spawn(async move {
                let _ = net_session.put(topic, payload).await;
            });
        }
        // On retourne le résultat (Succès ou Erreur) au contrôleur HTTP.
        update_result
    }
}

impl Handler<NetworkUpdate> for DocServer {
    type Result = ();

    fn handle(&mut self, msg: NetworkUpdate, _: &mut Context<Self>) {
        // 2. Petit affichage console pour vérifier que la réception marche (Debug).
        if let Ok(remote_doc) = serde_json::from_str::<Document>(&msg.payload) {
            // On insère (ou on écrase) le document dans notre base locale avec la nouvelle version
            // reçue du réseau. Notre réplique est désormais synchronisée avec les autres !
            println!(" Reçu via le Réseau (Topic: {}): {} (v{})", msg.key, remote_doc.name, remote_doc.version);
            self.repo.docs.insert(remote_doc.doc_id.clone(), remote_doc.clone());
        }
    }
}



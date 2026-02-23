use zenoh::Config;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
enum DocMessage {
    Create { client_id: String, doc_id: String, content: String },
    Update { client_id: String, old_version: String, new_version: String, content: String },
}


struct DocState {
    current_version: String,
    content: String,
    winning_client: String, 
    update_count: u32,
}

#[tokio::main]
async fn main() {
    println!(" [SERVEUR] Démarrage du nœud de stockage");
    
    let session = zenoh::open(Config::default()).await.unwrap();
    let subscriber = session.declare_subscriber("reseau/documents").await.unwrap();
    
    // Le "disque dur" de notre serveur pour stocker les documents
    let mut documents: HashMap<String, DocState> = HashMap::new();

    println!(" [SERVEUR] En attente de documents...\n");

    while let Ok(sample) = subscriber.recv_async().await {
        let payload_str = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
        
        if let Ok(msg) = serde_json::from_str::<DocMessage>(&payload_str) {
            match msg {
                DocMessage::Create { client_id, doc_id, content } => {
                    println!("📥 [CRÉATION par {}] ID: {} | Contenu: '{}'", client_id, doc_id, content);
                    
                    // On sauvegarde le document et on définit le créateur comme gagnant initial
                    documents.insert(doc_id.clone(), DocState {
                        current_version: doc_id,
                        content,
                        winning_client: client_id,
                        update_count: 0,
                    });
                },
                
                DocMessage::Update { client_id, old_version, new_version, content } => {
                    let base_doc_id = old_version.split('.').next().unwrap_or(&old_version).to_string();
                    println!("📥 [UPDATE reçu de {}] {} -> {}", client_id, old_version, new_version);

                    if let Some(state) = documents.get_mut(&base_doc_id) {
                        state.update_count += 1;

                        // 
                        if state.update_count > 1 {
                            println!("   ⚠️ [ALERTE CONCURRENCE] Il y a plus d'une mise à jour sur le doc_id '{}' !", base_doc_id);
                            
                            // Règle déterministe : Ordre lexicographique (Client_1 < Client_2)
                            if client_id <= state.winning_client {
                                println!("    [RÉSOLUTION] L'ID '{}' gagne (Priorité haute). Mise à jour ACCEPTÉE.", client_id);
                                state.current_version = new_version;
                                state.content = content;
                                state.winning_client = client_id.clone();
                            } else {
                                println!("    [RÉSOLUTION] L'ID '{}' perd face à '{}' (Priorité faible). Mise à jour REJETÉE.", client_id, state.winning_client);
                            }
                        } else {
                            
                            state.current_version = new_version;
                            state.content = content;
                            state.winning_client = client_id.clone();
                            println!("    [SUCCÈS] Première mise à jour acceptée.");
                        }
                    } else {
                        println!("   ⚠️ [ERREUR] Le document '{}' n'existe pas.", base_doc_id);
                    }
                }
            }
        }
    }
}
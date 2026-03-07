use zenoh::Config;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
enum DocMessage {
    Create { client_id: String, doc_id: String, content: String },
    Update { client_id: String, old_version: String, new_version: String, content: String },
    Notify { doc_id: String, content: String }, 
}

struct DocState {
    current_version: String,
    content: String,
    winning_client: String, 
    last_base_version: String, //  Permet de suivre l'historique pour les versions 1.2, 1.3, etc.
}

#[tokio::main]
async fn main() {
    println!(" [SERVEUR] Démarrage du nœud de stockage");
    
    let session = zenoh::open(Config::default()).await.unwrap();
    let subscriber = session.declare_subscriber("reseau/documents").await.unwrap();
    let mut documents: HashMap<String, DocState> = HashMap::new();

    println!(" [SERVEUR] En attente de documents...\n");

    while let Ok(sample) = subscriber.recv_async().await {
        let payload_str = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
        
        if let Ok(msg) = serde_json::from_str::<DocMessage>(&payload_str) {
            match msg {
                DocMessage::Create { client_id, doc_id, content } => {
                    println!(" [CRÉATION par {}] ID: {} | Contenu: '{}'", client_id, doc_id, content);
                    
                    documents.insert(doc_id.clone(), DocState {
                        current_version: doc_id.clone(),
                        content: content.clone(),
                        winning_client: client_id.clone(),
                        last_base_version: String::new(),
                    });

                    let notif = DocMessage::Notify { doc_id: doc_id.clone(), content: content.clone() };
                    session.put("reseau/notifications", serde_json::to_string(&notif).unwrap()).await.unwrap();
                },
                
                DocMessage::Update { client_id, old_version, new_version, content } => {
                    let base_doc_id = old_version.split('.').next().unwrap_or(&old_version).to_string();
                    println!("📥 [UPDATE reçu de {}] {} -> {}", client_id, old_version, new_version);

                    if let Some(state) = documents.get_mut(&base_doc_id) {
                        
                        // CAS 1 : C'est une suite logique normale (ex: on passe de 1.1 à 1.2)
                        if old_version == state.current_version {
                            println!("    ✅ [SUCCÈS] Mise à jour séquentielle acceptée.");
                            
                            state.last_base_version = state.current_version.clone();
                            state.current_version = new_version.clone();
                            state.content = content.clone();
                            state.winning_client = client_id.clone();

                            let notif = DocMessage::Notify { doc_id: new_version, content };
                            session.put("reseau/notifications", serde_json::to_string(&notif).unwrap()).await.unwrap();
                        
                        // CAS 2 : Conflit Concurrent ! Deux clients ont travaillé sur la même base.
                        } else if old_version == state.last_base_version {
                            println!("   ⚠️ [ALERTE CONCURRENCE] Modification concurrente détectée sur la base '{}' !", old_version);
                            
                            
                            if client_id < state.winning_client {
                                println!("    ✅ [RÉSOLUTION] L'ID '{}' a un rang supérieur. Il ÉCRASE la version de '{}'.", client_id, state.winning_client);
                                
                                state.current_version = new_version.clone();
                                state.content = content.clone();
                                state.winning_client = client_id.clone();
                                //  le Client 1 écrase le texte du Client 2 !
                                let notif = DocMessage::Notify { doc_id: new_version, content };
                                session.put("reseau/notifications", serde_json::to_string(&notif).unwrap()).await.unwrap();
                            } else {
                                println!("    ❌ [RÉSOLUTION] L'ID '{}' perd face à '{}' (Priorité faible). REJETÉ.", client_id, state.winning_client);
                            }
                        
                        // CAS 3 : La version du client est complètement dépassée
                        } else {
                            println!("    ❌ [ERREUR] Trop tard ! Le document est déjà à la version '{}'.", state.current_version);
                        }

                    } else {
                        println!("   ⚠️ [ERREUR] Le document '{}' n'existe pas.", base_doc_id);
                    }
                },
                DocMessage::Notify { .. } => {}
            }
        }
    }
}
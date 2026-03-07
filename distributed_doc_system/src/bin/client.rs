use std::env;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use zenoh::Config;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
enum DocMessage {
    Create { client_id: String, doc_id: String, content: String },
    Update { client_id: String, old_version: String, new_version: String, content: String },
    /// Notification diffusée par le serveur une fois le conflit résolu (Modèle Publish/Subscribe)
    Notify { doc_id: String, content: String }, 
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let id = if args.len() > 1 { args[1].clone() } else { "Client_Anonyme".to_string() };

    println!(" [{}] Nœud Client prêt", id);
    let session = zenoh::open(Config::default()).await.unwrap();

    let sub_session = session.clone();
    
    // On écoute UNIQUEMENT les notifications validées par le serveur
    tokio::spawn(async move {
        let subscriber = sub_session.declare_subscriber("reseau/notifications").await.unwrap();
        while let Ok(sample) = subscriber.recv_async().await {
            let payload_str = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
            
            if let Ok(msg) = serde_json::from_str::<DocMessage>(&payload_str) {
                //  doc_id et doc_content
                if let DocMessage::Notify { doc_id, content } = msg {
                    println!("\n [NOTIFICATION] Mise à jour validée par le réseau !");
                    println!("    doc_id: {}", doc_id);
                    println!("    doc_content: \"{}\"", content);
                }
            }
        }
    });

    let mut reader = BufReader::new(io::stdin());
    let mut line = String::new();

    println!("⌨️  Commandes disponibles :");
    println!("   - create <doc_id> \"<contenu>\"");
    println!("   - update <doc_id_version> <doc_id_new_version> \"<nouveau_contenu>\"\n");

    loop {
        line.clear();
        if reader.read_line(&mut line).await.unwrap() == 0 { break; }
        
        let input = line.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() { continue; }

        if parts[0] == "create" && parts.len() >= 3 {
            let doc_id = parts[1].to_string();
            let content = parts[2..].join(" ").replace("\"", "");
            
            let msg = DocMessage::Create { client_id: id.clone(), doc_id, content };
            let payload = serde_json::to_string(&msg).unwrap();
            session.put("reseau/documents", payload).await.unwrap();

        } else if parts[0] == "update" && parts.len() >= 4 {
            let old_version = parts[1].to_string();
            let new_version = parts[2].to_string();
            let content = parts[3..].join(" ").replace("\"", "");
            
            let msg = DocMessage::Update { client_id: id.clone(), old_version, new_version, content };
            let payload = serde_json::to_string(&msg).unwrap();
            session.put("reseau/documents", payload).await.unwrap();
            
        } else {
            println!("❌ Commande non reconnue. Vérifiez la syntaxe.");
        }
    }
}
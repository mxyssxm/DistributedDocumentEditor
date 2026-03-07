use actix_web::{web, App, HttpServer, HttpResponse, Responder, get, post, put};
use actix_files::Files;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use zenoh::Config;

#[derive(Serialize, Deserialize, Debug)]
enum DocMessage {
    Create { client_id: String, doc_id: String, content: String },
    Update { client_id: String, old_version: String, new_version: String, content: String },
    Notify { doc_id: String, content: String },
}

#[derive(Clone, Serialize)]
struct LocalDoc {
    doc_id: String,
    name: String,
    content: String,
    version: usize,
}

struct AppState {
    zenoh_session: zenoh::Session,
    docs: Arc<Mutex<HashMap<String, LocalDoc>>>,
}

#[derive(Deserialize)]
struct CreateReq { name: String }

#[derive(Deserialize)]
struct UpdateReq { content: String, version: usize }

#[get("/api/docs")]
async fn get_docs(state: web::Data<AppState>) -> impl Responder {
    let docs_map = state.docs.lock().await;
    let mut list: Vec<serde_json::Value> = vec![];
    for (id, doc) in docs_map.iter() {
        list.push(serde_json::json!({ "doc_id": id, "name": &doc.name }));
    }
    // 🌟 AJOUT : On trie la liste par ordre alphabétique pour que Doc1.1 soit sous Doc1
    list.sort_by(|a, b| a["name"].as_str().unwrap().cmp(b["name"].as_str().unwrap()));
    HttpResponse::Ok().json(list)
}

#[get("/api/docs/{id}")]
async fn get_doc(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let docs_map = state.docs.lock().await;
    if let Some(doc) = docs_map.get(&path.into_inner()) {
        HttpResponse::Ok().json(doc)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[post("/api/docs")]
async fn create_doc(req: web::Json<CreateReq>, state: web::Data<AppState>) -> impl Responder {
    let msg = DocMessage::Create {
        client_id: "Interface_Web".to_string(),
        doc_id: req.name.clone(),
        content: String::new(),
    };
    let payload = serde_json::to_string(&msg).unwrap();
    state.zenoh_session.put("reseau/documents", payload).await.unwrap();
    HttpResponse::Ok().json("Création envoyée")
}

#[put("/api/docs/{id}")]
async fn update_doc(path: web::Path<String>, req: web::Json<UpdateReq>, state: web::Data<AppState>) -> impl Responder {
    let doc_id = path.into_inner();
    let docs_map = state.docs.lock().await;
    
    if let Some(local_doc) = docs_map.get(&doc_id) {
        if req.version < local_doc.version {
            return HttpResponse::Conflict().body("Conflit ! Quelqu'un d'autre a modifié ce document.");
        }

        let old_v = if req.version == 0 { doc_id.clone() } else { format!("{}.{}", doc_id, req.version) };
        let new_v = format!("{}.{}", doc_id, req.version + 1);

        let msg = DocMessage::Update {
            client_id: "Interface_Web".to_string(),
            old_version: old_v,
            new_version: new_v,
            content: req.content.clone(),
        };
        let payload = serde_json::to_string(&msg).unwrap();
        state.zenoh_session.put("reseau/documents", payload).await.unwrap();
        HttpResponse::Ok().json(serde_json::json!({ "version": req.version + 1 }))
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 DÉMARRAGE DE L'INTERFACE WEB AVEC HISTORIQUE SUR LE PORT 8080...");

    let mut initial_map = HashMap::new();
    initial_map.insert("Doc_Demo".to_string(), LocalDoc {
        doc_id: "Doc_Demo".to_string(),
        name: "Document de Démo".to_string(),
        content: "Si tu vois ce texte, l'interface est bien connectée !".to_string(),
        version: 0,
    });
    
    let session = zenoh::open(Config::default()).await.unwrap();
    let docs_state = Arc::new(Mutex::new(initial_map));
    let docs_clone = docs_state.clone();
    let sub_session = session.clone();

    tokio::spawn(async move {
        let subscriber = sub_session.declare_subscriber("reseau/notifications").await.unwrap();
        while let Ok(sample) = subscriber.recv_async().await {
            let payload_str = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
            if let Ok(msg) = serde_json::from_str::<DocMessage>(&payload_str) {
                if let DocMessage::Notify { doc_id, content } = msg {
                    let parts: Vec<&str> = doc_id.split('.').collect();
                    let base_name = parts[0].to_string();
                    let version: usize = if parts.len() > 1 { parts[1].parse().unwrap_or(0) } else { 0 };

                    let mut map = docs_clone.lock().await;
                    
                    
                    map.insert(base_name.clone(), LocalDoc {
                        doc_id: base_name.clone(),
                        name: base_name.clone(),
                        content: content.clone(),
                        version,
                    });

                    
                    if version > 0 {
                        map.insert(doc_id.clone(), LocalDoc {
                            doc_id: doc_id.clone(),
                            name: format!("📜 {}", doc_id), 
                            content: content.clone(),
                            version,
                        });
                    }
                }
            }
        }
    });

    let app_state = web::Data::new(AppState {
        zenoh_session: session,
        docs: docs_state,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(get_docs)
            .service(get_doc)
            .service(create_doc)
            .service(update_doc)
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
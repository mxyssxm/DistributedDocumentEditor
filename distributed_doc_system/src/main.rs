use actix::Actor;
use actix_files as fs;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;

mod models;
mod services;
mod controllers;

use services::doc_server::DocServer;
use models::message::NetworkUpdate;
use controllers::api::*;

fn main() -> std::io::Result<()> {
    println!(" Démarrage du Nœud ");

    let zenoh_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let (session_arc, subscriber) = zenoh_rt.block_on(async {
        let config = zenoh::Config::default();
        let session = zenoh::open(config).await.unwrap();
        let session_arc = Arc::new(session);
        let subscriber = session_arc.declare_subscriber("docs/**").await.unwrap();
        
        (session_arc, subscriber)
    });

    actix_web::rt::System::new().block_on(async move {
        let server = DocServer::new(session_arc.clone()).start();
        let server_addr = server.clone();

        actix::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                let key = sample.key_expr().to_string();
                let payload_bytes = sample.payload().to_bytes();
                if let Ok(payload) = String::from_utf8(payload_bytes.to_vec()) {
                    server_addr.do_send(NetworkUpdate { key, payload });
                }
            }
        });

       // On écoute sur le port 0 pour avoir un port dynamique
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        println!("===================================================");
        println!(" Nœud en ligne sur : http://127.0.0.1:{}", port);
        println!(" Ouvre ce lien dans ton navigateur !");
        println!("===================================================");

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(server.clone()))
                .route("/api/docs", web::get().to(get_docs))
                .route("/api/docs", web::post().to(create_doc))
                .route("/api/docs/{id}", web::get().to(get_doc))
                .route("/api/docs/{id}", web::put().to(update_doc))
                .service(fs::Files::new("/", "./static").index_file("index.html"))
        })
        .listen(listener)? // On utilise le "listener" au lieu de "bind"
        .run() //ici on utilise run()
        .await
    })
}

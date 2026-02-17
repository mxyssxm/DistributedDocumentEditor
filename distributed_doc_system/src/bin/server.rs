use zenoh::Config;

#[tokio::main]
async fn main() {
    println!(" [SERVEUR] Démarrage du Serveur Zenoh pur (POC)...");
    let session = zenoh::open(Config::default()).await.unwrap();
    let subscriber = session.declare_subscriber("reseau/valeurs").await.unwrap();
    println!(" [SERVEUR] En attente de valeurs des clients...\n");

    while let Ok(sample) = subscriber.recv_async().await {
        let valeur = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
        println!(" [SERVEUR] Valeur reçue : '{}'", valeur);
    }
}
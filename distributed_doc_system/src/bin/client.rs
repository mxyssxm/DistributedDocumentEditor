use std::env;
use std::time::Duration;
use zenoh::Config;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let id = if args.len() > 1 { args[1].clone() } else { "Client_Anonyme".to_string() };

    println!(" [{}] Démarrage du Client Zenoh pur...", id);
    let session = zenoh::open(Config::default()).await.unwrap();
    let mut compteur = 1;

    loop {
        let valeur = format!("Donnée #{} provenant de {}", compteur, id);
        println!("📤 [{}] Envoi de la valeur : '{}'", id, valeur);
        session.put("reseau/valeurs", valeur).await.unwrap();

        compteur += 1;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
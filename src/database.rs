use std::{io::Result, sync::Arc};

use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client, Collection, Database,
};
use serde_json::Value;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub struct MongoDBClient {
    conn: Client,
    db: Database,
    pub servers: Collection<Document>,
}

impl MongoDBClient {
    pub async fn new() -> Arc<Mutex<Self>> {
        let client_options = ClientOptions::parse("mongodb://mse_mongodb:27017")
            .await
            .unwrap();
        let client = Client::with_options(client_options).unwrap();

        let db = client.database("minecraft_search_engine");
        let collection = db.collection("servers");

        Arc::new(Mutex::new(MongoDBClient {
            conn: client,
            db,
            servers: collection,
        }))
    }

    pub async fn add(&self, info: &Value) -> Result<()> {
        let bson_doc = mongodb::bson::to_document(info).expect("Failed to convert JSON to BSON");

        self.servers.insert_one(bson_doc).await.unwrap();
        Ok(())
    }

    pub async fn get_ips(&self) -> Result<Vec<(String, String)>> {
        let mut cursor = self.servers.find(doc! {}).await.unwrap();

        let mut results: Vec<(String, String)> = Vec::new();
        while cursor.advance().await.unwrap() {
            let val = serde_json::to_value(&cursor.current()).unwrap();
            results.push((val["ip"].as_str().unwrap_or("localhost").to_string(), val["port"].as_str().unwrap_or("25565").to_string()));
        }

        Ok(results)
    }
}

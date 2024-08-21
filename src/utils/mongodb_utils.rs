use crate::models::config_models::Config;
use mongodb::bson::Document;
use mongodb::Collection;
use mongodb::{bson::doc, options::ClientOptions, Client};
use rocket::futures::TryStreamExt;
use std::env;
use std::env::VarError;
use std::io::Error;

fn load_mongo_url() -> Result<String, VarError> {
    env::var("MONGODB_URL")
}

async fn load_mongo_client() -> mongodb::error::Result<Client> {
    let mongodb_url_result = load_mongo_url();

    let mongodb_url = match mongodb_url_result {
        Err(_) => panic!("Unable to read MONGODB_URL from .env"),
        Ok(mongodb_url) => mongodb_url,
    };

    let client_options = ClientOptions::parse(&mongodb_url).await?;

    Client::with_options(client_options)
}

async fn mongo_find(
    collection: Collection<Document>,
    filter: Document,
) -> mongodb::error::Result<Vec<Document>> {
    collection.find(filter).await?.try_collect::<Vec<_>>().await
}

// async fn load_mongo_config() -> Result<Config, Error> {
//     let client = load_mongo_client()
//         .await
//         .expect("Unable to connect to client");
//
//     let home_config = client.database("HomeConfig");
//
//     let relays = mongo_find(home_config.collection("Relays"), doc! {})
//         .await
//         .expect("Unable to find relays from database");
//     let presets = mongo_find(home_config.collection("Presets"), doc! {})
//         .await
//         .expect("Unable to find presets from database");
//
//
//     // Config { relays, presets }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::relays::RelayType::KasaMultiPlug;
    use crate::models::relays::{KasaPlug, MongodbRelay};
    use dotenv::dotenv;

    #[test]
    fn test_loading_mongo_url_from_dotenv() {
        dotenv().ok();
        let url = load_mongo_url();
        assert!(url.is_ok())
    }

    #[tokio::test]
    async fn test_connecting_to_mongo_db() {
        dotenv().ok();
        let client = load_mongo_client().await;
        assert!(client.is_ok())
    }

    #[tokio::test]
    async fn test_getting_config_from_mongodb() {
        dotenv().ok();
        let client_result = load_mongo_client().await;
        assert!(client_result.is_ok());

        let client = client_result.unwrap();

        let home_config = client.database("HomeConfig");

        let relays_collection: Collection<Document> = home_config.collection("Relays");
        let filter = doc! {};
        let query_result = relays_collection.find(filter).await;
        assert!(query_result.is_ok());

        assert!(query_result.unwrap().try_collect::<Vec<_>>().await.is_ok());
    }

    #[tokio::test]
    async fn test_getting_config_from_mongodb_relays() {
        dotenv().ok();
        let client_result = load_mongo_client().await;
        assert!(client_result.is_ok());

        let client = client_result.unwrap();

        let home_config = client.database("HomeConfig");

        let relays_collection: Collection<MongodbRelay> = home_config.collection("Relays");
        let filter = doc! {};
        let query_result = relays_collection.find(filter).await;
        assert!(query_result.is_ok());

        let relay_query = query_result
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .expect("Could not parse Relay Query");

        // let relays: Vec<KasaPlug> = relay_query
        //     .iter()
        //     .filter_map(|relay: &MongodbRelay| match relay.relay_type.as_str() {
        //         "KasaPlug" => Some(KasaPlug::new(
        //             relay.ip.clone(),
        //             relay.name.clone(),
        //             relay.room.clone(),
        //         )),
        //         _ => None,
        //     })
        //     .collect();

        let relays: Vec<KasaPlug> = relay_query
            .iter()
            .filter_map(|relay: &MongodbRelay| match &relay.relay_type {
                MongodbRelay::KasaPlug => Some(KasaPlug::new(
                    relay.ip.clone(),
                    relay.name.clone(),
                    relay.room.clone(),
                )),
                // RelayType::KasaMultiPlug => Some(KasaPlugMulti)
                _ => None,
            })
            .collect();

        // println!("{:?}", relays);
    }
}

use crate::models::config_models::Config;
use crate::models::presets::Preset;
use crate::models::relays::{ConfigRelay, ConfigRelayType, KasaMultiPlug, KasaPlug, RelayType};
use dotenv::dotenv;
use mongodb::{bson::doc, options::ClientOptions, Client};
use mongodb::{Collection, Database};
use rocket::futures::TryStreamExt;
use std::collections::HashMap;
use std::env;
use std::env::VarError;

fn load_mongo_url() -> Result<String, VarError> {
    dotenv().ok();
    env::var("MONGODB_URL")
}

async fn load_mongo_client() -> mongodb::error::Result<Client> {
    let mongodb_url = load_mongo_url().expect("Unable to read MONGODB_URL from .env");
    let client_options = ClientOptions::parse(&mongodb_url).await?;
    Client::with_options(client_options)
}

async fn find_mongo_relays(
    database: &Database,
) -> Result<HashMap<String, RelayType>, mongodb::error::Error> {
    let relays_collection: Collection<ConfigRelay> = database.collection("Relays");
    let filter = doc! {};
    let query_result = relays_collection.find(filter).await;
    let relay_query = query_result?.try_collect::<Vec<_>>().await?;

    let mut relays: HashMap<String, RelayType> = HashMap::new();

    for relay in relay_query {
        match &relay.relay_type {
            ConfigRelayType::KasaPlug => {
                let mut plug =
                    KasaPlug::new(relay.ip.clone(), relay.name.clone(), relay.room.clone());

                if plug.connected().is_ok() {
                    relays.insert(plug.name.clone(), RelayType::KasaPlug(plug));
                }
            }
            ConfigRelayType::KasaMultiPlug => {
                let plugs =
                    KasaMultiPlug::new(relay.ip.clone(), relay.names.clone(), relay.room.clone());

                for mut plug in plugs {
                    if plug.connected().is_ok() {
                        relays.insert(plug.name.clone(), RelayType::KasaMultiPlug(plug));
                    }
                }
            }
        }
    }

    Ok(relays)
}

async fn find_mongo_presets(
    database: &Database,
) -> Result<HashMap<String, Preset>, mongodb::error::Error> {
    let presets_collection: Collection<Preset> = database.collection("Presets");
    let filter = doc! {};
    let query_result = presets_collection.find(filter).await;

    let preset_query = query_result?.try_collect::<Vec<_>>().await?;

    let mut presets: HashMap<String, Preset> = HashMap::new();

    for preset in preset_query {
        presets.insert(preset.name.clone(), preset);
    }

    presets
        .entry("Custom".to_string())
        .or_insert_with(|| Preset {
            name: "Custom".to_string(),
            enabled: false,
            relays: HashMap::new(),
        });

    presets
        .entry("FullOff".to_string())
        .or_insert_with(|| Preset {
            name: "FullOff".to_string(),
            enabled: false,
            relays: HashMap::new(),
        });

    Ok(presets)
}

pub async fn load_mongo_config() -> Result<Config, mongodb::error::Error> {
    let client = load_mongo_client()
        .await
        .expect("Unable to connect to client");

    let home_config = client.database("HomeConfig");

    let relays = find_mongo_relays(&home_config).await?;
    let presets = find_mongo_presets(&home_config).await?;

    Ok(Config { relays, presets })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use mongodb::bson::Document;

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

        let relays = find_mongo_relays(&home_config)
            .await
            .expect("Could not get Mongo relays");
    }

    #[tokio::test]
    async fn test_getting_config_from_mongodb_presets() {
        dotenv().ok();
        let client_result = load_mongo_client().await;
        assert!(client_result.is_ok());

        let client = client_result.unwrap();

        let home_config = client.database("HomeConfig");

        let presets = find_mongo_presets(&home_config)
            .await
            .expect("Could not get Mongo presets");
    }
}

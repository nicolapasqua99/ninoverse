use std::env::{self, VarError};

use rdkafka::admin::{AdminOptions, TopicReplication};

use crate::kafka_handler::structs::KafkaNinoverseTopic;

pub fn test_configuration() -> Result<(), VarError> {
    dotenv::dotenv().ok();
    env::var("SELF_PORT").expect("SELF_PORT configuration value missing");
    env::var("PG_HOST").expect("PG_HOST configuration value missing");
    env::var("PG_PORT").expect("PG_PORT configuration value missing");
    env::var("PG_USER").expect("PG_USER configuration value missing");
    env::var("PG_PASSWORD").expect("PG_PASSWORD configuration value missing");
    env::var("PG_DB").expect("PG_DB configuration value missing");
    env::var("KAFKA_BROKER").expect("KAFKA_BROKER configuration value missing");
    env::var("KAFKA_TOPIC").expect("KAFKA_TOPIC configuration value missing");
    Ok(())
}

pub fn get_self_port() -> String {
    env::var("SELF_PORT").unwrap_or_else(|_| "7878".to_string())
}

pub fn get_pg_host() -> String {
    env::var("PG_HOST").unwrap_or_else(|_| "postgres".to_string())
}

pub fn get_pg_port() -> String {
    env::var("PG_PORT").unwrap_or_else(|_| "5432".to_string())
}

pub fn get_pg_user() -> String {
    env::var("PG_USER").unwrap_or_else(|_| "nino".to_string())
}

pub fn get_pg_password() -> String {
    env::var("PG_PASSWORD").unwrap_or_else(|_| "nino".to_string())
}

pub fn get_pg_db() -> String {
    env::var("PG_DB").unwrap_or_else(|_| "ninoverse".to_string())
}

pub fn get_kafka_generic_broker() -> String {
    env::var("KAFKA_BROKER").unwrap_or_else(|_| "broker:9092".to_string())
}

pub fn get_kafka_generic_topic<'a>() -> Vec<KafkaNinoverseTopic<'a>> {
    let topic_config = env::var("KAFKA_TOPIC").unwrap_or_else(|_| "ninoverse:1:1".to_string());
    let topic_config_array: Vec<&str> = topic_config.split(":").collect();
    let topic_name = topic_config_array
        .get(0)
        .unwrap_or(&"ninoverse")
        .to_string();
    let partition = topic_config_array
        .get(1)
        .unwrap_or(&"1")
        .parse::<i32>()
        .unwrap_or(1);
    let offset = topic_config_array
        .get(2)
        .unwrap_or(&"1")
        .parse::<i64>()
        .unwrap_or(1);
    vec![KafkaNinoverseTopic {
        topic: topic_name.clone(),
        partition,
        offset,
        replication: TopicReplication::Fixed(0),
        config: vec![],
    }]
}

pub fn get_kafka_admin_options() -> AdminOptions {
    AdminOptions::new()
}

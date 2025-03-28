pub mod structs;

use chrono::DateTime;

use rdkafka::{
    ClientConfig, Message,
    admin::{AdminClient, NewTopic, TopicReplication},
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
    producer::{FutureProducer, FutureRecord},
};

use futures::TryStreamExt;

use structs::KafkaNinoverseBrokerContext;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    KafkaChannelMessage,
    configuration_handler::{
        get_kafka_admin_options, get_kafka_generic_broker, get_kafka_generic_topic,
    },
};

async fn create_kafka_consumer() -> Result<StreamConsumer, rdkafka::error::KafkaError> {
    println!("CONSUMER_CREATION: Creating consumer.");
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "ninoverse")
        .set("bootstrap.servers", get_kafka_generic_broker())
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        // .set("auto.offset.reset", "earliest")
        .create()
        .expect("CONSUMER_CREATION: creation failed");
    println!("CONSUMER_CREATION: Created, subscribing to topic ninoverse.");
    consumer
        .subscribe(&["ninoverse"])
        .expect("CONSUMER_CREATION: Can't subscribe to specified topic");
    println!("CONSUMER_CREATION: Subscribed to topic ninoverse.");
    Ok(consumer)
}

async fn create_kafka_producer() -> Result<FutureProducer, rdkafka::error::KafkaError> {
    println!("PRODUCER_CREATION: Creating producer.");
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", get_kafka_generic_broker())
        .set("message.timeout.ms", "5000")
        .create()
        .expect("PRODUCER_CREATION: Producer creation error");
    println!("PRODUCER_CREATION: Producer created.");
    Ok(producer)
}

async fn handle_kafka_message(message: rdkafka::message::OwnedMessage) -> Result<(), KafkaError> {
    let mut key = String::new();
    if let Some(key_bytes) = message.key() {
        key = String::from_utf8(key_bytes.to_vec())
            .unwrap_or_else(|_| "MESSAGE: Invalid UTF-8 array for key value.".to_string());
    };
    let timestamp = DateTime::from_timestamp(
        message
            .timestamp()
            .to_millis()
            .expect("MESSAGE: Incorrect timestamp format.")
            / 1000,
        0,
    )
    .expect("MESSAGE: Error creating date string.");
    let mut payload = String::new();
    if let Some(payload_bytes) = message.payload() {
        payload = String::from_utf8(payload_bytes.to_vec())
            .unwrap_or_else(|_| "MESSAGE: Invalid UTF-8 array for payload value.".to_string());
    };
    println!(
        "MESSAGE: {}:{}/{}/{}/{}: {}",
        message.topic(),
        message.partition(),
        message.offset(),
        key.as_str(),
        timestamp.to_string().as_str(),
        payload
    );
    Ok(())
}

async fn init_kafka_consumer(kafka_thread_channel_sender: Sender<KafkaChannelMessage>) {
    let consumer = create_kafka_consumer()
        .await
        .expect("Error in creating consumer");
    println!("CONSUMER: Thread started, creating stream and consuming it.");
    kafka_thread_channel_sender
        .send(KafkaChannelMessage::KafkaConsumerStarted)
        .await
        .expect("CONSUMER: Error in sending message to producer thread");
    consumer
        .stream()
        .try_for_each(|borrowed_message| handle_kafka_message(borrowed_message.detach()))
        .await
        .expect("CONSUMER: Error in consuming stream");
}

async fn init_kafka_producer(mut kafka_thread_receiver: Receiver<KafkaChannelMessage>) {
    let producer = create_kafka_producer()
        .await
        .expect("Error in creating producer");
    loop {
        println!("PRODUCER: Thread waiting for consumer to start.");
        kafka_thread_receiver.recv().await.inspect(|message| {
            if let KafkaChannelMessage::KafkaConsumerError = message {
                println!("PRODUCER: Received an error in consumer start.")
            }
        });
        break;
    }
    println!("PRODUCER: Thread started, sending messages.");
    while let Some(received) = kafka_thread_receiver.recv().await {
        if let KafkaChannelMessage::Message { sender, content } = received {
            let queue_timeout = std::time::Duration::from_secs(1);
            let key = sender;
            let payload = format!("Message: {}", content);
            let record = FutureRecord::to("ninoverse")
                .payload(payload.as_str())
                .key(key.as_str());
            producer
                .send(record, queue_timeout)
                .await
                .expect("Error in sending message");
        }
    }
}

async fn create_kafka_admin_client() -> Result<AdminClient<KafkaNinoverseBrokerContext>, KafkaError>
{
    println!("ADMIN_CLIENT_CREATION: Creating admin client.");
    let admin_client: AdminClient<KafkaNinoverseBrokerContext> = ClientConfig::new()
        .set("bootstrap.servers", get_kafka_generic_broker())
        .create_with_context(KafkaNinoverseBrokerContext {})
        .expect("Failed to create AdminClient with custom context");
    println!("ADMIN_CLIENT_CREATION: Created Admin client.");
    Ok(admin_client)
}

async fn init_kafka_topics(admin_client: AdminClient<KafkaNinoverseBrokerContext>) {
    println!("TOPIC_CREATION: Creating topics object.");
    let kafka_topics = get_kafka_generic_topic();
    let kafka_new_topics: Vec<NewTopic<'_>> = kafka_topics
        .iter()
        .map(|element| NewTopic {
            name: &element.topic,
            num_partitions: element.partition,
            replication: TopicReplication::Fixed(1),
            config: vec![],
        })
        .collect();
    if kafka_new_topics.iter().len() == 0 {
        println!("TOPIC_CREATION: No topic created (no topic creation requested).");
    } else {
        let options = get_kafka_admin_options();
        println!("TOPIC_CREATION: Sending request to Kafka Admin Client");
        let topic_creation_result_list = admin_client
            .create_topics(&kafka_new_topics, &options)
            .await
            .expect("TOPIC_CREATION: Topic creation failed");
        for topic_creation_result in topic_creation_result_list {
            println!("TOPIC_CREATION: {:?}", topic_creation_result);
        }
    }
}

pub async fn init_kafka(
    kafka_thread_sender: Sender<KafkaChannelMessage>,
    kafka_thread_receiver: Receiver<KafkaChannelMessage>,
) {
    let admin_client = create_kafka_admin_client()
        .await
        .expect("INIT_KAFKA: Admin client creation failed");
    init_kafka_topics(admin_client).await;
    let consumer_handle = tokio::spawn(async {
        init_kafka_consumer(kafka_thread_sender).await;
    });
    let producer_handle = tokio::spawn(async {
        init_kafka_producer(kafka_thread_receiver).await;
    });
    tokio::try_join!(producer_handle, consumer_handle).expect("Error in Kafka threads");
}

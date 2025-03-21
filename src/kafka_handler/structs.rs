use rdkafka::{admin::TopicReplication, ClientContext};

#[allow(dead_code)]
pub struct KafkaNinoverseTopic<'a> {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub replication: TopicReplication<'a>,
    pub config: Vec<(&'a str, &'a str)>,
}

pub struct KafkaNinoverseBrokerContext {}

impl ClientContext for KafkaNinoverseBrokerContext {
    const ENABLE_REFRESH_OAUTH_TOKEN: bool = false;

    // Provided methods
    // fn log(&self, level: RDKafkaLogLevel, fac: &str, log_message: &str) {}
    // fn stats(&self, statistics: Statistics) {}
    // fn stats_raw(&self, statistics: &[u8]) {}
    // fn error(&self, error: KafkaError, reason: &str) {}
    // fn generate_oauth_token(
    //     &self,
    //     _oauthbearer_config: Option<&str>,
    // ) -> Result<OAuthToken, Box<dyn Error>> {  }
}

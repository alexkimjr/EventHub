use crate::config::KafkaConfig;
use kafka::producer::{Producer, Record, RequiredAcks};
use std::sync::mpsc::{self, Sender};
use std::thread;

/// Non-blocking Kafka producer backed by a background thread and an unbounded channel.
pub struct KafkaProducer {
    sender: Sender<String>,
}

impl KafkaProducer {
    pub fn new(cfg: &KafkaConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let brokers: Vec<String> = cfg.brokers.split(',').map(|s| s.trim().to_string()).collect();
        let topic = cfg.topic.clone();

        let (tx, rx) = mpsc::channel::<String>();

        // spawn background thread that owns the synchronous producer
        thread::spawn(move || {
            // attempt to create producer here; if it fails we log and exit thread
            match Producer::from_hosts(brokers)
                .with_ack_timeout(std::time::Duration::from_secs(5))
                .with_required_acks(RequiredAcks::One)
                .create()
            {
                Ok(mut producer) => {
                    for msg in rx {
                        let record = Record::from_value(&topic, msg.into_bytes());
                        if let Err(e) = producer.send(&record) {
                            tracing::error!(error = ?e, "kafka send failed in background thread");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = ?e, "failed to create kafka producer in background thread");
                }
            }
        });

        Ok(Self { sender: tx })
    }

    
    pub fn send_nonblocking(&self, payload: String) -> Result<(), Box<dyn std::error::Error>> {
        self.sender
            .send(payload)
            .map_err(|e| format!("send queue closed: {}", e).into())
    }
}

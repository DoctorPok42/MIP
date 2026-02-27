use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

use crate::server::frame::Frame;

pub type SharedBroker = Arc<Mutex<Broker>>;

pub struct Broker {
    pub clients: HashMap<u64, UnboundedSender<Frame>>,
    pub topics: HashMap<String, HashSet<u64>>,
}

impl Broker {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            topics: HashMap::new(),
        }
    }

    pub fn client_tx(&self, client_id: u64) -> Option<&UnboundedSender<Frame>> {
        self.clients.get(&client_id)
    }

    pub fn register_client(&mut self, client_id: u64, tx: UnboundedSender<Frame>) {
        self.clients.insert(client_id, tx);
    }

    pub fn unregister_client(&mut self, client_id: u64) {
        self.clients.remove(&client_id);

        for subscribers in self.topics.values_mut() {
            subscribers.remove(&client_id);
        }
    }

    pub fn subscribe(&mut self, client_id: u64, topic: String) {
        self.topics.entry(topic).or_default().insert(client_id);
    }

    pub fn unsubscribe(&mut self, client_id: u64, topic: &str) {
        if let Some(subscribers) = self.topics.get_mut(topic) {
            subscribers.remove(&client_id);
        }
    }

    pub fn publish(&self, topic: &str, frame: Frame) {

        if let Some(subscribers) = self.topics.get(topic) {
            for client_id in subscribers {
                if let Some(tx) = self.clients.get(client_id) {
                    let _ = tx.send(frame.clone());
                }
            }
        }
    }
}

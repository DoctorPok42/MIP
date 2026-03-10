use crate::protocol::{FrameFlags, FrameType, Header, MessageKind};
use crate::server::broker::{SharedBroker};
use crate::server::frame::Frame;

pub struct ConnectionContext {
    pub client_id: u64,
    pub subscriptions: Vec<String>,
}

pub struct Dispatcher;

impl Dispatcher {
    pub async fn dispatch(
        broker: SharedBroker,
        context: &mut ConnectionContext,
        frame: Frame,
    ) -> Option<Frame> {
        match frame.header.frame_type {
            FrameType::Hello => Self::handle_hello(broker, context, frame).await,

            FrameType::Ping => Some(Self::handle_ping(frame)),

            FrameType::Subscribe => Self::handle_subscribe(broker, context, frame).await,

            FrameType::Unsubscribe => Self::handle_unsubscribe(broker, context, frame).await,

            FrameType::Publish => Self::handle_publish(broker, frame).await,

            _ => None,
        }
    }

    async fn handle_hello(
      broker: SharedBroker,
      context: &mut ConnectionContext,
      frame: Frame,
    ) -> Option<Frame> {
        let mut broker_session = broker.lock().await;

        let client_id = if frame.payload.len() >= 8 {
            u64::from_be_bytes(frame.payload[0..8].try_into().ok()?)
        } else {
            // Generate a new client_id
            use std::sync::atomic::{AtomicU64, Ordering};
            static CLIENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
            CLIENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
        };

        context.client_id = client_id;

        // Check if a session already exists for this client_id
        if let Some(session) = broker_session.sessions.get(&client_id) {
            println!("Resuming session for client {}", client_id);

            let topics = session.topics.clone();

            // Restore subscriptions
            for topic in topics {
                broker_session.subscribe(client_id, topic);
            }
        } else {
            println!("Creating new session for client {}", client_id);

            // Create a new session
            broker_session.sessions.insert(client_id, super::Session {
                topics: Vec::new(),
            });
        }

        Some(Frame {
            header: Header::new(
                FrameType::Ack,
                MessageKind::State,
                8,
                frame.header.msg_id,
                FrameFlags::empty()
            ),
            payload: client_id.to_be_bytes().to_vec(),
        })
    }

    fn handle_ping(frame: Frame) -> Frame {
        Frame {
            header: Header::new(
                FrameType::Pong,
                MessageKind::Event,
                0,
                frame.header.msg_id,
                FrameFlags::empty(),
            ),
            payload: Vec::new(),
        }
    }

    async fn handle_subscribe(
        broker: SharedBroker,
        context: &mut ConnectionContext,
        frame: Frame,
    ) -> Option<Frame> {
        let topic = String::from_utf8(frame.payload).ok()?;

        if !frame.header.flags.contains(FrameFlags::ACK_REQUIRED) {
            return None;
        }

        {
            let mut broker = broker.lock().await;
            broker.subscribe(context.client_id, topic.clone());

            if let Some(session) = broker.sessions.get_mut(&context.client_id) {
                if !session.topics.contains(&topic) {
                    session.topics.push(topic.clone());
                }
            }
        }

        context.subscriptions.push(topic);

        Some(Frame {
            header: Header::new(
                FrameType::Ack,
                MessageKind::State,
                0,
                frame.header.msg_id,
                FrameFlags::empty(),
            ),
            payload: Vec::new(),
        })
    }

    async fn handle_unsubscribe(
        broker: SharedBroker,
        context: &mut ConnectionContext,
        frame: Frame,
    ) -> Option<Frame> {
        let topic = String::from_utf8(frame.payload).ok()?;

        if !frame.header.flags.contains(FrameFlags::ACK_REQUIRED) {
            return None;
        }

        {
            let mut broker = broker.lock().await;
            broker.unsubscribe(context.client_id, &topic);

            if let Some(session) = broker.sessions.get_mut(&context.client_id) {
                session.topics.retain(|t| t != &topic);
            }
        }

        context.subscriptions.retain(|t| t != &topic);

        Some(Frame {
            header: Header::new(
                FrameType::Ack,
                MessageKind::State,
                0,
                frame.header.msg_id,
                FrameFlags::empty(),
            ),
            payload: Vec::new(),
        })
    }

    async fn handle_publish(broker: SharedBroker, frame: Frame) -> Option<Frame> {
        if frame.payload.len() < 2 {
            return None;
        }

        let topic_len = u16::from_be_bytes([frame.payload[0], frame.payload[1]]) as usize;

        if frame.payload.len() < 2 + topic_len {
            return None;
        }

        let topic = match std::str::from_utf8(&frame.payload[2..2 + topic_len]) {
            Ok(t) => t,
            Err(_) => return None,
        };

        let outgoing = Frame {
            header: frame.header.clone(),
            payload: frame.payload.clone(),
        };

        let broker = broker.lock().await;
        broker.publish(topic, outgoing);

        Some(Frame {
            header: Header::new(
                FrameType::Ack,
                MessageKind::State,
                0,
                frame.header.msg_id,
                FrameFlags::empty(),
            ),
            payload: Vec::new(),
        })
    }
}

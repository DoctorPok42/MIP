use crate::protocol::{FrameFlags, FrameType, Header, MessageKind};
use crate::server::broker::SharedBroker;
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
            FrameType::Ping => Some(Self::handle_ping(frame)),

            FrameType::Subscribe => Self::handle_subscribe(broker, context, frame).await,

            FrameType::Unsubscribe => Self::handle_unsubscribe(broker, context, frame).await,

            FrameType::Publish => {
                Self::handle_publish(broker, frame).await;
                None
            }

            FrameType::Close => None,

            _ => None,
        }
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

        {
            let mut broker = broker.lock().await;
            broker.subscribe(context.client_id, topic.clone());
        }

        if !frame.header.flags.contains(FrameFlags::ACK_REQUIRED) {
            return None;
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

        {
            let mut broker = broker.lock().await;
            broker.unsubscribe(context.client_id, &topic);
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

    async fn handle_publish(broker: SharedBroker, frame: Frame) {
        if frame.payload.len() < 2 {
            return;
        }

        let topic_len = u16::from_be_bytes([frame.payload[0], frame.payload[1]]) as usize;

        if frame.payload.len() < 2 + topic_len {
            return;
        }

        let topic = match std::str::from_utf8(&frame.payload[2..2 + topic_len]) {
            Ok(t) => t,
            Err(_) => return,
        };

        let outgoing = Frame {
            header: frame.header.clone(),
            payload: frame.payload.clone(),
        };

        let broker = broker.lock().await;
        broker.publish(topic, outgoing);
    }
}

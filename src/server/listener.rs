use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

use crate::protocol::FrameType;
use crate::server::broker::{Broker, SharedBroker};
use crate::server::dispatcher::{ConnectionContext, Dispatcher};
use crate::server::frame::Frame;

pub struct Listener;

impl Listener {
    pub async fn start(addr: &str) -> tokio::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        let broker: SharedBroker = Arc::new(Mutex::new(Broker::new()));

        println!("MSIP server listening on {}", addr);

        loop {
            let (socket, _) = listener.accept().await?;
            let broker = broker.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(socket, broker).await {
                    eprintln!("Client error: {}", e);
                }
            });
        }
    }

    async fn handle_client(socket: TcpStream, broker: SharedBroker) -> tokio::io::Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel::<Frame>();

        let mut context = ConnectionContext {
            client_id: 0,
            subscriptions: Vec::new(),
        };

        let (mut reader, mut writer) = socket.into_split();

        tokio::spawn(async move {
            while let Some(frame) = rx.recv().await {
                if let Err(e) = frame.write_to(&mut writer).await {
                    eprintln!("Error writing frame: {}", e);
                    break;
                }
            }
        });

        loop {
            let frame = match Frame::read_from(&mut reader).await {
                Ok(f) => f,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::ConnectionReset {
                        break;
                    }
                    eprintln!("Error reading frame: {}", e);
                    break;
                }
            };

            if matches!(frame.header.frame_type, FrameType::Close) {
                break;
            }

            if let Some(response) = Dispatcher::dispatch(broker.clone(), &mut context, frame).await
            {
                let client_id = context.client_id;
                if client_id != 0 {
                    let mut broker_lock = broker.lock().await;

                    if !broker_lock.clients.contains_key(&client_id) {
                        broker_lock.register_client(client_id, tx.clone());
                    }

                    if let Some(client_tx) = broker_lock.client_tx(client_id) {
                        let _ = client_tx.send(response);
                    } else {
                        eprintln!("Client {} not found for response", client_id);
                        break;
                    }
                } else {
                    let _ = tx.send(response);
                }
            } else {
                let payload = "Invalid frame or unsupported operation".as_bytes().to_vec();
                let _ = tx.send(Frame {
                    payload: payload.clone(),
                    header: crate::protocol::Header::new(
                        FrameType::Error,
                        crate::protocol::MessageKind::Event,
                        payload.len() as u32,
                        0,
                        crate::protocol::FrameFlags::empty(),
                    ),
                });
                break;
            }
        }

        {
            let mut broker = broker.lock().await;
            if context.client_id != 0 {
                broker.unregister_client(context.client_id);
            }
        }

        Ok(())
    }
}

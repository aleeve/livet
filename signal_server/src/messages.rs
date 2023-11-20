use protocol::{ClientCommand, ServerCommand};

use axum::extract::ws::Message;
use tokio::sync::mpsc;
use tracing::log::{log, Level};

use sqlx::{Pool, Row, Sqlite};
use std::net::SocketAddr;
use std::ops::ControlFlow;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum BroadcastCommand {
    HelloFrom {
        uuid: uuid::Uuid,
        tx: mpsc::Sender<DirectCommand>,
    },
    GoodbyFrom {
        uuid: uuid::Uuid,
    },
}

#[derive(Clone, Debug)]
pub enum DirectCommand {
    WelcomeFrom {
        uuid: uuid::Uuid,
        tx: mpsc::Sender<DirectCommand>,
    },

    CreateOfferFor {
        uuid: uuid::Uuid,
    },

    CreateAnswerFor {
        uuid: uuid::Uuid,
        offer: String,
    },

    GetAnswerFrom {
        uuid: uuid::Uuid,
        answer: String,
    },

    GetIceFrom {
        uuid: uuid::Uuid,
        ice: String,
    },
}

pub fn process_message(msg: Message) -> ControlFlow<(), ClientCommand> {
    match msg {
        Message::Text(t) => match serde_json::from_str::<ClientCommand>(&t) {
            Ok(command) => ControlFlow::Continue(command),
            Err(e) => {
                log!(Level::Error, "Failed to parse message: {}", e);
                ControlFlow::Break(())
            }
        },

        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> sent close with code {} and reason `{}`",
                    cf.code, cf.reason
                );
            } else {
                println!(">>>  somehow sent close message without CloseFrame");
            }
            ControlFlow::Break(())
        }

        _ => {
            log!(Level::Error, "Got unsuported message");
            ControlFlow::Break(())
        }
    }
}

/// See https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs for original
///
use protocol::{ClientCommand, ServerCommand};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    headers,
    response::IntoResponse,
    TypedHeader,
};
use tokio::sync::mpsc::{channel, Sender};
use tracing::log::{log, Level};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use uuid::Uuid;

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

use crate::{
    messages::{process_message, BroadcastCommand, DirectCommand},
    ServerState,
};

pub async fn ws_handler(
    State(server_state): State<ServerState>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    log!(Level::Info, "`{}` at {} connected.", user_agent, addr);
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(server_state, socket, addr))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut server_state: ServerState, mut socket: WebSocket, who: SocketAddr) {
    let my_uuid = Uuid::new_v4();
    let db = server_state.db.clone();

    // Client must specify what session it wants to join
    let tx_session = server_state.join_session(String::from("hej")).await;

    // If allowed client will introduce itself to the memebers of the session, allowing them direct
    // serverside communication.
    let (tx_direct, mut rx_direct) = channel(5);
    let hello = BroadcastCommand::HelloFrom {
        uuid: my_uuid.clone(),
        tx: tx_direct.clone(),
    };
    if tx_session.send(hello).is_err() {
        log!(Level::Info, "{who} is first to join");
    }

    // Start listening for session and socket updates
    let mut rx_session = tx_session.subscribe();
    let introduction = DirectCommand::WelcomeFrom {
        uuid: my_uuid.clone(),
        tx: tx_direct.clone(),
    };
    let mut participants = HashMap::<Uuid, Sender<DirectCommand>>::new();
    loop {
        tokio::select! {
            // Deal with incoming messages from the client
            Some(command) = socket.recv() => {
                match command {
                    Ok(msg)  => {
                        log!(Level::Info, "Got message {}", msg.to_text().unwrap());
                        match process_message(msg) {
                            ControlFlow::Break(()) => break,
                            ControlFlow::Continue(command) => {
                                match command {
                                    ClientCommand::Offer(uuid, offer) => {
                                        if let Some(tx) = participants.get(&uuid) {
                                            let tx = tx.clone();
                                            tokio::spawn( async move {
                                                tx.send(DirectCommand::CreateAnswerFor { uuid: my_uuid, offer}).await;
                                            });
                                        } else {
                                            log!(Level::Warn, "Missing participant");
                                        }

                                    },
                                    ClientCommand::Answer(uuid, answer) => {
                                        let tx = participants.get(&uuid).expect(&format!("Missing session participant {uuid}")).clone();
                                        tokio::spawn( async move {
                                            tx.send(DirectCommand::GetAnswerFrom { uuid: my_uuid, answer }).await;
                                        });
                                    }
                                    ClientCommand::IceCandidate(uuid, ice) => {
                                        let tx = participants.get(&uuid).expect(&format!("Missing session participant {uuid}")).clone();
                                        tokio::spawn( async move {
                                            tx.send(DirectCommand::GetIceFrom { uuid: my_uuid, ice}).await;
                                        });
                                    }
                                }
                            }
                        }
                    },
                    Err(_) => {
                        log!(Level::Warn, "Socket closed");
                        break;
                    }
                }
            }

            // Keep track of session members
            Ok(command) = rx_session.recv() => {
                match command {
                    BroadcastCommand::HelloFrom{uuid ,tx} => {
                        tx.send(introduction.clone()).await;
                        participants.insert(uuid, tx);
                        let polite = true;
                        socket.send(Message::Text(serde_json::to_string(&ServerCommand::AddMember(uuid.clone(), polite)).unwrap())).await;
                        socket.send(Message::Text(serde_json::to_string(&ServerCommand::CreateOffer(uuid.clone())).unwrap())).await;
                    }
                    BroadcastCommand::GoodbyFrom{uuid} => {
                        socket.send(Message::Text(serde_json::to_string(&ServerCommand::DropMember(uuid)).unwrap())).await;
                        participants.remove(&uuid);
                    }
                }
            }

            // Let others trigger outgoing traffic to client
            Some(command) = rx_direct.recv() => {
                match command {
                    DirectCommand::WelcomeFrom { uuid, tx} => {
                        participants.insert(uuid, tx);
                        let polite = false;
                        socket.send(Message::Text(serde_json::to_string(&ServerCommand::AddMember(uuid.clone(), polite)).unwrap())).await;
                    },
                    DirectCommand::CreateOfferFor { uuid } => {
                        let command = ServerCommand::CreateOffer(uuid);
                        socket.send(Message::Text(serde_json::to_string(&command).unwrap())).await;
                    },
                    DirectCommand::CreateAnswerFor { uuid, offer} => {
                        let command = ServerCommand::CreateAnswer(uuid, offer);
                        socket.send(Message::Text(serde_json::to_string(&command).unwrap())).await;
                    },

                    DirectCommand::GetAnswerFrom { uuid, answer} => {
                        let command = ServerCommand::GetAnswer(uuid, answer);
                        socket.send(Message::Text(serde_json::to_string(&command).unwrap())).await;
                    },

                    DirectCommand::GetIceFrom { uuid, ice} => {
                        let command = ServerCommand::AddIceCandidate(uuid, ice);
                        socket.send(Message::Text(serde_json::to_string(&command).unwrap())).await;
                    },
                }
            }

        }
    }
    tx_session.send(BroadcastCommand::GoodbyFrom { uuid: my_uuid });
    log!(Level::Info, "Websocket context {who} destroyed");
}

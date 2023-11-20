use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ServerCommand {
    CreateOffer(Uuid),
    CreateAnswer(Uuid, String),
    GetAnswer(Uuid, String),
    AddIceCandidate(Uuid, String),
    AddMember(Uuid, bool),
    DropMember(Uuid),
}

impl ServerCommand {
    pub fn get_uuid(&self) -> Uuid {
        match self {
            ServerCommand::CreateOffer(uuid) => uuid.clone(),
            ServerCommand::CreateAnswer(uuid, _) => uuid.clone(),
            ServerCommand::GetAnswer(uuid, _) => uuid.clone(),
            ServerCommand::AddMember(uuid, _) => uuid.clone(),
            ServerCommand::DropMember(uuid) => uuid.clone(),
            ServerCommand::AddIceCandidate(uuid, _) => uuid.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ClientCommand {
    Offer(Uuid, String),
    Answer(Uuid, String),
    IceCandidate(Uuid, String),
}

#[derive(Serialize, Deserialize)]
pub struct Musician {
    pub id: i32,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Band {
    pub id: i32,
    pub name: Option<String>,
    pub member: Vec<Musician>,
}

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub id: i32,
    pub name: Option<String>,
    pub member: Vec<Musician>,
}

use crate::model::*;

use geng::prelude::*;

pub type ClientId = i64;

pub type ClientConnection = geng::net::client::Connection<ServerMessage, ClientMessage>;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Ping,
    Error(String),
    // YourToken(String),
    RoomJoined(RoomInfo),
    StartGame(GameRole),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Pong,
    // Login(String),
    CreateRoom,
    JoinRoom(String),
    SelectRole(GameRole),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomInfo {
    pub code: String,
    pub players: Vec<ClientId>,
}

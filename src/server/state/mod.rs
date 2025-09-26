use super::*;

use geng::prelude::{rand::prelude::Distribution, *};

pub struct Client {
    pub sender: Box<dyn geng::net::Sender<ServerMessage>>,
    // pub token: String,
    pub room: Option<Arc<str>>,
}

pub struct ServerState {
    pub timer: Timer,
    next_id: ClientId,
    clients: HashMap<ClientId, Client>,
    rooms: HashMap<Arc<str>, Room>,
}

pub struct Room {
    pub code: Arc<str>,
    pub players: Vec<ClientId>,
}

impl Room {
    pub fn new(code: Arc<str>, player: ClientId) -> Self {
        Self {
            code,
            players: vec![player],
        }
    }

    pub fn info(&self) -> RoomInfo {
        RoomInfo {
            code: self.code.to_string(),
            players: self.players.clone(),
        }
    }

    pub fn player_join(&mut self, player: ClientId) {
        self.players.push(player);
    }
}

impl ServerState {
    pub const TICKS_PER_SECOND: f32 = 2.0;

    pub fn new() -> Self {
        Self {
            timer: Timer::new(),
            next_id: 1,
            clients: HashMap::new(),
            rooms: HashMap::new(),
        }
    }

    pub fn client_connect(
        &mut self,
        mut sender: Box<dyn geng::net::Sender<ServerMessage>>,
    ) -> ClientId {
        if self.clients.is_empty() {
            self.timer.reset();
        }

        let my_id = self.next_id;
        self.next_id += 1;

        sender.send(ServerMessage::Ping);
        // let token = rand::distributions::Alphanumeric.sample_string(&mut thread_rng(), 16);
        // sender.send(ServerMessage::YourToken(token.clone()));

        let client = Client {
            sender,
            // token,
            room: None,
        };

        self.clients.insert(my_id, client);
        my_id
    }

    pub fn client_disconnect(&mut self, client_id: ClientId) {
        let _client = self.clients.remove(&client_id).unwrap();
    }

    pub fn handle_message(&mut self, client_id: ClientId, message: ClientMessage) {
        let client = self
            .clients
            .get_mut(&client_id)
            .expect("Sender not found for client");
        match message {
            ClientMessage::Pong => {
                // client.sender.send(ServerMessage::Time(
                //     state.timer.elapsed().as_secs_f64() as f32
                // ));
                client.sender.send(ServerMessage::Ping);
            }
            ClientMessage::CreateRoom => {
                match &client.room {
                    Some(_) => {
                        // The client already has a room, there's some desync
                        // TODO: fix desync
                    }
                    None => 'room: {
                        for _ in 0..10 {
                            let code: String =
                                rand::distributions::Uniform::new_inclusive('A', 'B')
                                    .sample_iter(&mut thread_rng())
                                    .take(4)
                                    .collect();
                            let code: Arc<str> = code.into();
                            if let std::collections::hash_map::Entry::Vacant(e) =
                                self.rooms.entry(code.clone())
                            {
                                client.room = Some(code.clone());
                                let room = Room::new(code, client_id);
                                client.sender.send(ServerMessage::RoomJoined(room.info()));
                                e.insert(room);
                                break 'room;
                            }
                        }
                        // Failed to create a room
                        // TODO: idk
                    }
                }
            }
            ClientMessage::JoinRoom(code) => {
                let code: Arc<str> = code.to_uppercase().into();
                if let Some(room) = self.rooms.get_mut(&code) {
                    if room.players.len() < 2 {
                        room.player_join(client_id);
                        client.sender.send(ServerMessage::RoomJoined(room.info()));
                    } else {
                        client
                            .sender
                            .send(ServerMessage::Error("room already full".into()));
                    }
                } else {
                    client
                        .sender
                        .send(ServerMessage::Error("non-existent room code".into()));
                }
            }
        }
    }

    pub fn tick(&mut self) {
        self.rooms.retain(|_code, room| {
            room.players.retain(|id| self.clients.contains_key(id));
            !room.players.is_empty()
        })
    }
}

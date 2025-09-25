use super::*;

use geng::prelude::*;

pub struct Client {
    pub sender: Box<dyn geng::net::Sender<ServerMessage>>,
}

pub struct ServerState {
    pub timer: Timer,
    pub next_id: ClientId,
    pub clients: HashMap<ClientId, Client>,
}

impl ServerState {
    pub const TICKS_PER_SECOND: f32 = 2.0;

    pub fn new() -> Self {
        Self {
            timer: Timer::new(),
            next_id: 1,
            clients: HashMap::new(),
        }
    }

    pub fn client_disconnect(&mut self, client_id: ClientId) {}

    pub fn handle_message(&mut self, client_id: ClientId, message: ClientMessage) {
        match message {
            ClientMessage::Pong => {
                let client = self
                    .clients
                    .get_mut(&client_id)
                    .expect("Sender not found for client");
                // client.sender.send(ServerMessage::Time(
                //     state.timer.elapsed().as_secs_f64() as f32
                // ));
                client.sender.send(ServerMessage::Ping);
            }
        }
    }

    pub fn tick(&mut self) {}
}

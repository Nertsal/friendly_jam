mod connection;
mod state;

use self::{connection::ClientConnection, state::*};

use crate::{interop::*, model::*};

use geng::prelude::*;

pub struct App {
    state: Arc<Mutex<ServerState>>,
    #[allow(dead_code)]
    background_thread: std::thread::JoinHandle<()>,
}

impl App {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(ServerState::new()));
        Self {
            state: state.clone(),
            background_thread: std::thread::spawn(move || {
                loop {
                    state.lock().unwrap().tick();
                    std::thread::sleep(std::time::Duration::from_secs_f32(
                        1.0 / ServerState::TICKS_PER_SECOND,
                    ));
                }
            }),
        }
    }
}

impl geng::net::server::App for App {
    type Client = ClientConnection;

    type ServerMessage = ServerMessage;

    type ClientMessage = ClientMessage;

    fn connect(
        &mut self,
        mut sender: Box<dyn geng::net::Sender<Self::ServerMessage>>,
    ) -> Self::Client {
        let mut state = self.state.lock().unwrap();
        if state.clients.is_empty() {
            state.timer.reset();
        }

        let my_id = state.next_id;
        state.next_id += 1;

        sender.send(ServerMessage::Ping);
        // let token = Alphanumeric.sample_string(&mut thread_rng(), 16);
        // sender.send(ServerMessage::YourToken(token.clone()));

        let client = Client {
            // token,
            sender,
        };

        state.clients.insert(my_id, client);
        ClientConnection {
            id: my_id,
            state: self.state.clone(),
        }
    }
}

mod connection;
mod state;

use self::{connection::ClientConnection, state::*};

use crate::interop::*;

use geng::prelude::*;

pub struct App {
    state: Arc<Mutex<ServerState>>,
    #[allow(dead_code)]
    background_thread: std::thread::JoinHandle<()>,
}

impl App {
    pub fn new(test: bool) -> Self {
        let state = Arc::new(Mutex::new(ServerState::new(test)));
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

    fn connect(&mut self, sender: Box<dyn geng::net::Sender<Self::ServerMessage>>) -> Self::Client {
        let mut state = self.state.lock().unwrap();
        let my_id = state.client_connect(sender);
        ClientConnection {
            id: my_id,
            state: self.state.clone(),
        }
    }
}

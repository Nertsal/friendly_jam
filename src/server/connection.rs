use super::*;

pub struct ClientConnection {
    pub id: ClientId,
    pub state: Arc<Mutex<ServerState>>,
}

impl Drop for ClientConnection {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        let _client = state.clients.remove(&self.id).unwrap();
        state.client_disconnect(self.id);
    }
}

impl geng::net::Receiver<ClientMessage> for ClientConnection {
    fn handle(&mut self, message: ClientMessage) {
        let mut state = self.state.lock().unwrap();
        state.handle_message(self.id, message);
    }
}

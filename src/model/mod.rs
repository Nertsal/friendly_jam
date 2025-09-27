use geng::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameRole {
    Dispatcher,
    Solver,
}

#[derive(Serialize, Deserialize)]
pub struct DispatcherState {
    pub door_sign_open: bool,
}

impl DispatcherState {
    pub fn new() -> Self {
        Self {
            door_sign_open: false,
        }
    }
}

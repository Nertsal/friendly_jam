use geng::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameRole {
    Dispatcher,
    Solver,
}

pub struct Model {}

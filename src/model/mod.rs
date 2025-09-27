mod collider;

pub use self::collider::*;

use geng::prelude::*;

pub type FCoord = R32;
pub type FTime = R32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameRole {
    Dispatcher,
    Solver,
}

#[derive(Serialize, Deserialize)]
pub struct DispatcherState {
    pub door_sign_open: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SolverState {}

impl DispatcherState {
    pub fn new() -> Self {
        Self {
            door_sign_open: false,
        }
    }
}

impl SolverState {
    pub fn new() -> Self {
        Self {}
    }
}

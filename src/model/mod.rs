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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatcherState {
    pub button_station_open: bool,
    pub door_sign_open: bool,
    pub monitor_unlocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverState {
    pub current_level: usize,
    pub levels_completed: usize,
}

impl DispatcherState {
    pub fn new() -> Self {
        Self {
            button_station_open: false,
            door_sign_open: false,
            monitor_unlocked: false,
        }
    }
}

impl SolverState {
    pub fn new() -> Self {
        Self {
            current_level: 0,
            levels_completed: 0,
        }
    }

    pub fn is_exit_open(&self) -> bool {
        self.current_level < self.levels_completed
    }
}

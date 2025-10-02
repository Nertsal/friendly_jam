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
    pub trashcan_evil: bool,
    pub solved_bubble_code: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub collider: Collider,
    pub velocity: vec2<FCoord>,
    pub state: PlayerState,
    pub control_timeout: Option<FTime>,
    pub facing_left: bool,
    pub can_hold_jump: bool,
    pub coyote_time: Option<FTime>,
    pub jump_buffer: Option<FTime>,
    pub animation_time: FTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerState {
    Grounded,
    Airborn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerAnimationState {
    Idle,
    Running,
    Jumping,
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
            trashcan_evil: true,
            solved_bubble_code: false,
        }
    }

    pub fn is_exit_open(&self) -> bool {
        self.current_level < self.levels_completed
    }
}

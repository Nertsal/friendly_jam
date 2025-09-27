use super::*;

use crate::model::{FCoord, FTime};

use geng_utils::key::EventKey;

#[derive(geng::asset::Load)]
pub struct SolverAssets {
    pub controls: SolverControls,
    pub rules: SolverRules,
}

#[derive(geng::asset::Load, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct SolverControls {
    pub move_left: Vec<EventKey>,
    pub move_right: Vec<EventKey>,
    pub jump: Vec<EventKey>,
}

#[derive(geng::asset::Load, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct SolverRules {
    pub buffer_time: FTime,
    pub coyote_time: FTime,
    pub gravity: vec2<FCoord>,
    pub fall_multiplier: FCoord,
    pub free_fall_speed: FCoord,
    pub low_multiplier: FCoord,
    pub move_speed: FCoord,
    pub acceleration_ground: FCoord,
    pub acceleration_air: FCoord,
    pub deceleration_ground: FCoord,
    pub deceleration_air: FCoord,
    pub jump_push: FCoord,
    pub jump_strength: FCoord,
}

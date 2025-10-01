use super::*;

use crate::model::{Collider, FCoord, FTime};

use geng_utils::key::EventKey;

#[derive(geng::asset::Load)]
pub struct SolverAssets {
    pub controls: SolverControls,
    pub rules: SolverRules,
    pub sprites: SolverSprites,
    #[load(listed_in = "_list.ron")]
    pub levels: Vec<SolverLevel>,
}

#[derive(geng::asset::Load)]
pub struct SolverSprites {
    pub level_bounds: Rc<PixelTexture>,
    pub door_open: Rc<PixelTexture>,
    pub door_closed: Rc<PixelTexture>,
    pub platform: Rc<PixelTexture>,
    pub player: SolverPlayerSprites,
    pub level1: Rc<PixelTexture>,
    pub fish: Rc<PixelTexture>,
    pub cinder_block: Rc<PixelTexture>,
}

#[derive(geng::asset::Load)]
pub struct SolverPlayerSprites {
    #[load(list = "0..=1")]
    pub idle: Vec<Rc<PixelTexture>>,
    #[load(list = "0..=3")]
    pub running: Vec<Rc<PixelTexture>>,
    #[load(list = "0..=1")]
    pub jump: Vec<Rc<PixelTexture>>,
}

#[derive(geng::asset::Load, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct SolverControls {
    pub move_left: Vec<EventKey>,
    pub move_right: Vec<EventKey>,
    pub jump: Vec<EventKey>,
    pub pickup: Vec<EventKey>,
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

#[derive(geng::asset::Load, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct SolverLevel {
    pub door_entrance: bool,
    pub door_exit: bool,
    pub spawnpoint: vec2<FCoord>,
    pub transition: Aabb2<FCoord>,
    #[serde(default)]
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub items: Vec<SolverItem>,
}

#[derive(Serialize, Deserialize)]
pub struct Platform {
    pub pos: vec2<FCoord>,
    pub width: FCoord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverItem {
    pub kind: SolverItemKind,
    #[serde(default)]
    pub pushable: bool,
    #[serde(default)]
    pub can_pickup: bool,
    #[serde(default)]
    pub has_gravity: bool,
    pub collider: Collider,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SolverItemKind {
    Fish,
    CinderBlock,
}

impl SolverSprites {
    pub fn item_texture(&self, kind: SolverItemKind) -> &Rc<PixelTexture> {
        match kind {
            SolverItemKind::Fish => &self.fish,
            SolverItemKind::CinderBlock => &self.cinder_block,
        }
    }
}

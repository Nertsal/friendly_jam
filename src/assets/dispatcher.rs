use super::*;

#[derive(geng::asset::Load)]
pub struct DispatcherAssets {
    pub sprites: DispatcherSprites,
    pub level: DispatcherLevel,
}

#[derive(geng::asset::Load)]
pub struct DispatcherSprites {
    pub sign_open: PixelTexture,
    pub sign_closed: PixelTexture,
    pub table: PixelTexture,
    pub monitor: PixelTexture,
    pub arrow_left: PixelTexture,
    pub arrow_right: PixelTexture,
}

#[derive(geng::asset::Load, Clone)]
pub struct DispatcherLevel {
    pub front: DispatcherView,
    pub left: DispatcherView,
    pub right: DispatcherView,
    pub back: DispatcherView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatcherViewSide {
    Front,
    Left,
    Right,
    Back,
}

#[derive(geng::asset::Load, Serialize, Deserialize, Clone)]
#[load(serde = "ron")]
pub struct DispatcherView {
    pub items: Vec<(DispatcherItem, DispatcherItemPosition)>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub enum DispatcherItem {
    DoorSign,
    Table,
    Monitor,
}

/// Positioning in screen-space with fixed 1920x1080 resolution.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DispatcherItemPosition {
    pub anchor: vec2<f32>,
    #[serde(default = "default_alignment")]
    pub alignment: vec2<f32>,
    pub size: Option<vec2<f32>>,
    #[serde(skip, default = "default_target")]
    pub hitbox: Aabb2<f32>,
}

fn default_alignment() -> vec2<f32> {
    vec2(0.5, 0.5)
}

fn default_target() -> Aabb2<f32> {
    Aabb2::ZERO
}

impl DispatcherLevel {
    pub fn get_side_mut(&mut self, side: DispatcherViewSide) -> &mut DispatcherView {
        match side {
            DispatcherViewSide::Front => &mut self.front,
            DispatcherViewSide::Left => &mut self.left,
            DispatcherViewSide::Right => &mut self.right,
            DispatcherViewSide::Back => &mut self.back,
        }
    }
}

impl DispatcherViewSide {
    pub fn cycle_left(self) -> Self {
        match self {
            Self::Front => Self::Left,
            Self::Left => Self::Back,
            Self::Back => Self::Right,
            Self::Right => Self::Front,
        }
    }

    pub fn cycle_right(self) -> Self {
        match self {
            Self::Front => Self::Right,
            Self::Left => Self::Front,
            Self::Back => Self::Left,
            Self::Right => Self::Back,
        }
    }
}

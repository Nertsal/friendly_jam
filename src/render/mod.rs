pub mod mask;
pub mod texture_atlas;
pub mod util;

use crate::context::*;

use geng::prelude::*;

pub type Color = Rgba<f32>;

pub struct GameRender {
    context: Context,
}

impl GameRender {
    pub fn new(context: &Context) -> Self {
        Self {
            context: context.clone(),
        }
    }
}

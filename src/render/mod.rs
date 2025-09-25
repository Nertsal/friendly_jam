use crate::context::*;

use geng::prelude::*;

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

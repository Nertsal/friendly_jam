use crate::context::Context;

use geng::prelude::*;

pub struct Game {
    context: Context,
}

impl Game {
    pub async fn new(context: &Context, connect: Option<String>) -> Self {
        Self {
            context: context.clone(),
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
    }
}

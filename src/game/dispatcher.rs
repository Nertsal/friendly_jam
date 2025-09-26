use super::*;

pub struct GameDispatcher {
    context: Context,
}

impl GameDispatcher {
    pub fn new(context: &Context) -> Self {
        Self {
            context: context.clone(),
        }
    }
}

impl geng::State for GameDispatcher {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
    }
}

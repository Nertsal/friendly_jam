use super::*;

const SCREEN_SIZE: vec2<usize> = vec2(1920, 1080);

pub struct GameDispatcher {
    context: Context,

    final_texture: ugli::Texture,
    screen: Aabb2<f32>,
}

impl GameDispatcher {
    pub fn new(context: &Context) -> Self {
        Self {
            context: context.clone(),

            final_texture: geng_utils::texture::new_texture(context.geng.ugli(), SCREEN_SIZE),
            screen: Aabb2::ZERO.extend_positive(vec2(1.0, 1.0)),
        }
    }

    fn draw_game(&mut self) {
        let framebuffer = &mut geng_utils::texture::attach_texture(
            &mut self.final_texture,
            self.context.geng.ugli(),
        );
        ugli::clear(
            framebuffer,
            Some(self.context.assets.palette.background),
            None,
            None,
        );
    }
}

impl geng::State for GameDispatcher {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
        self.draw_game();
        let draw = geng_utils::texture::DrawTexture::new(&self.final_texture)
            .fit_screen(vec2(0.5, 0.5), framebuffer);
        self.screen = draw.target;
        draw.draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);
    }
}

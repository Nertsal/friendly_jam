use super::*;

use crate::render::texture_atlas::SubTexture;

pub struct ButtonWidget {
    pub state: WidgetState,
    pub texture: SubTexture,
}

impl ButtonWidget {
    pub fn new(texture: SubTexture) -> Self {
        Self {
            state: WidgetState::new(),
            texture,
        }
    }

    pub fn layout(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }
}

impl Widget for ButtonWidget {
    crate::simple_widget_state!();

    fn draw(&self, context: &UiContext) -> Geometry {
        context.geometry.texture(
            self.state.position,
            mat3::identity(),
            Rgba::WHITE,
            &self.texture,
        )
    }
}

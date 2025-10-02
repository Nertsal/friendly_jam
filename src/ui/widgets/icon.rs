use super::*;

use crate::render::texture_atlas::SubTexture;

#[derive(Clone)]
pub struct IconWidget {
    pub state: WidgetState,
    pub texture: SubTexture,
    pub color: Rgba<f32>,
}

impl IconWidget {
    pub fn new(texture: SubTexture) -> Self {
        Self {
            state: WidgetState::new(),
            texture: texture.clone(),
            color: Rgba::WHITE,
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }
}

impl Widget for IconWidget {
    crate::simple_widget_state!();
    fn draw(&self, context: &UiContext) -> Geometry {
        if !self.state.visible {
            return Geometry::new();
        }

        context.geometry.texture(
            self.state.position,
            mat3::identity(),
            self.color,
            &self.texture,
        )
    }
}

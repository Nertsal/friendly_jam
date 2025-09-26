use super::*;

use crate::render::texture_atlas::SubTexture;

pub struct ButtonWidget {
    pub state: WidgetState,
    pub texture: SubTexture,
    pub text: TextWidget,
}

impl ButtonWidget {
    pub fn new(texture: SubTexture) -> Self {
        Self {
            state: WidgetState::new(),
            texture,
            text: TextWidget::new(""),
        }
    }

    pub fn with_text(mut self, text: impl Into<Text>) -> Self {
        self.text.text = text.into();
        self
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        self.text.update(position, context);
    }
}

impl Widget for ButtonWidget {
    crate::simple_widget_state!();

    fn draw(&self, context: &UiContext) -> Geometry {
        let mut geometry = Geometry::new();
        geometry.merge(context.geometry.texture(
            self.state.position,
            mat3::identity(),
            Rgba::WHITE,
            &self.texture,
        ));
        geometry.merge(self.text.draw(context));
        geometry
    }
}

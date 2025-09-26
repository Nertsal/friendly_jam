use crate::render::util::TextRenderOptions;

use super::*;

pub struct TextWidget {
    pub state: WidgetState,
    pub text: Text,
    pub options: TextRenderOptions,
}

impl TextWidget {
    pub fn new(text: impl Into<Text>) -> Self {
        Self {
            state: WidgetState::new(),
            text: text.into(),
            options: TextRenderOptions {
                size: 1000.0,
                ..default()
            },
        }
    }

    pub fn align(&mut self, align: vec2<f32>) {
        self.options.align = align;
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }

    pub fn draw_colored(&self, context: &UiContext, color: Rgba<f32>) -> Geometry {
        let font = &context.font;
        let measure = font.measure(&self.text, 1.0);

        let size = self.state.position.size();
        let right = vec2(size.x, 0.0).rotate(self.options.rotation).x;
        let left = vec2(0.0, size.y).rotate(self.options.rotation).x;
        let width = if left.signum() != right.signum() {
            left.abs() + right.abs()
        } else {
            left.abs().max(right.abs())
        };

        let max_width = width * 0.9; // Leave some space TODO: move into a parameter or smth
        let max_size = max_width / measure.width();
        let size = self.options.size.min(max_size);

        let mut options = self.options;
        options.size = size;
        options.color = color;

        context.geometry.text(
            self.text.clone(),
            geng_utils::layout::aabb_pos(self.state.position, options.align),
            options,
        )
    }
}

impl Widget for TextWidget {
    crate::simple_widget_state!();

    fn draw(&self, context: &UiContext) -> Geometry {
        self.draw_colored(context, self.options.color)
    }
}

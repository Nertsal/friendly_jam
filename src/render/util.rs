use crate::{render::mask::MaskedStack, ui::Geometry};

use super::*;

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub fn get_pixel_scale(framebuffer_size: vec2<usize>) -> f32 {
    const TARGET_SIZE: vec2<usize> = vec2(640, 360);
    let size = framebuffer_size.as_f32();
    let ratio = size / TARGET_SIZE.as_f32();
    ratio.x.min(ratio.y)
}

#[derive(Debug, Clone, Copy)]
pub struct TextRenderOptions {
    pub size: f32,
    pub align: vec2<f32>,
    pub color: Color,
    pub hover_color: Color,
    pub press_color: Color,
    pub rotation: Angle,
}

#[derive(Debug, Clone, Copy)]
pub struct DashRenderOptions {
    pub width: f32,
    pub dash_length: f32,
    pub space_length: f32,
}

impl TextRenderOptions {
    pub fn new(size: f32) -> Self {
        Self { size, ..default() }
    }

    // pub fn size(self, size: f32) -> Self {
    //     Self { size, ..self }
    // }

    pub fn align(self, align: vec2<f32>) -> Self {
        Self { align, ..self }
    }

    pub fn color(self, color: Color) -> Self {
        Self { color, ..self }
    }
}

impl Default for TextRenderOptions {
    fn default() -> Self {
        Self {
            size: 1.0,
            align: vec2::splat(0.5),
            color: Color::WHITE,
            hover_color: Color {
                r: 0.7,
                g: 0.7,
                b: 0.7,
                a: 1.0,
            },
            press_color: Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            },
            rotation: Angle::ZERO,
        }
    }
}

pub struct UtilRender {
    context: Context,
    pub unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
}

impl UtilRender {
    pub fn new(context: Context) -> Self {
        Self {
            unit_quad: geng_utils::geometry::unit_quad_geometry(context.geng.ugli()),
            context,
        }
    }

    pub fn draw_geometry(
        &self,
        masked: &mut MaskedStack,
        geometry: Geometry,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        // log::debug!("Rendering geometry:");
        // log::debug!("^- triangles: {}", geometry.triangles.len() / 3);

        let framebuffer_size = framebuffer.size().as_f32();

        // Masked
        let mut frame = masked.pop_mask();
        for masked_geometry in geometry.masked {
            let mut masking = frame.start();
            self.draw_geometry(masked, masked_geometry.geometry, camera, &mut masking.color);
            masking.mask_quad(masked_geometry.clip_rect);
            frame.draw(
                masked_geometry.z_index,
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::straight_alpha()),
                    depth_func: Some(ugli::DepthFunc::Less),
                    ..default()
                },
                framebuffer,
            );
        }
        masked.return_mask(frame);

        // Text
        for text in geometry.text {
            self.draw_text_with(
                text.text,
                text.position,
                text.z_index,
                text.options,
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::straight_alpha()),
                    depth_func: Some(ugli::DepthFunc::LessOrEqual),
                    ..default()
                },
                camera,
                framebuffer,
            );
        }

        // Triangles & Textures
        let triangles =
            ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), geometry.triangles);
        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.texture_ui,
            ugli::DrawMode::Triangles,
            &triangles,
            (
                ugli::uniforms! {
                    u_texture: self.context.assets.atlas.texture(),
                    u_model_matrix: mat3::identity(),
                    u_color: Color::WHITE,
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_text_with(
        &self,
        text: impl AsRef<str>,
        position: vec2<impl Float>,
        z_index: f32,
        mut options: TextRenderOptions,
        params: ugli::DrawParameters,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let text = text.as_ref();
        let font = &self.context.assets.font;
        let framebuffer_size = framebuffer.size().as_f32();

        let position = position.map(Float::as_f32);
        let position = crate::util::world_to_screen(camera, framebuffer_size, position);

        let scale = crate::util::world_to_screen(
            camera,
            framebuffer_size,
            vec2::splat(std::f32::consts::FRAC_1_SQRT_2),
        ) - crate::util::world_to_screen(camera, framebuffer_size, vec2::ZERO);
        options.size *= scale.len();
        let font_size = options.size * 0.6; // TODO: could rescale all dependent code but whatever

        let mut position = position;
        for line in text.lines() {
            let measure = font.measure(line, font_size);
            let size = measure.size();
            let align = size * (options.align - vec2::splat(0.5)); // Centered by default
            let descent = -font.descent() * font_size;
            let align = vec2(
                measure.center().x + align.x,
                descent + (measure.max.y - descent) * options.align.y,
            );

            let transform = mat3::translate(position)
                * mat3::rotate(options.rotation)
                * mat3::translate(-align);

            font.draw_with(
                framebuffer,
                line,
                z_index,
                font_size,
                options.color,
                transform,
                params.clone(),
            );
            position.y -= options.size; // NOTE: larger than text size to space out better
        }
    }
}

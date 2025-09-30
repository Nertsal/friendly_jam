use geng::prelude::*;

pub fn world_to_screen(
    camera: &impl geng::AbstractCamera2d,
    framebuffer_size: vec2<f32>,
    pos: vec2<f32>,
) -> vec2<f32> {
    let pos = (camera.projection_matrix(framebuffer_size) * camera.view_matrix()) * pos.extend(1.0);
    let pos = pos.xy() / pos.z;
    vec2(
        (pos.x + 1.0) / 2.0 * framebuffer_size.x,
        (pos.y + 1.0) / 2.0 * framebuffer_size.y,
    )
}

pub fn wrap_text(font: &crate::assets::Font, text: &str, target_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for source_line in text.trim().lines() {
        let mut line = String::new();
        for word in source_line.split_whitespace() {
            if line.is_empty() {
                line += word;
                continue;
            }
            if font.measure(&(line.clone() + " " + word), 1.0).width() * 0.6 > target_width {
                lines.push(line);
                line = word.to_string();
            } else {
                line += " ";
                line += word;
            }
        }
        lines.push(line);
    }
    lines
}

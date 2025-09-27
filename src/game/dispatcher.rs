use super::*;

use crate::{assets::*, model::DispatcherState};

use geng_utils::conversions::Vec2RealConversions;

const SCREEN_SIZE: vec2<usize> = vec2(1920, 1080);

pub struct GameDispatcher {
    context: Context,

    final_texture: ugli::Texture,
    framebuffer_size: vec2<usize>,
    screen: Aabb2<f32>,
    /// Default scaling from texture to SCREEN_SIZE.
    texture_scaling: f32,

    cursor_position_raw: vec2<f64>,
    cursor_position_game: vec2<f32>,

    client_state: DispatcherStateClient,
    state: DispatcherState,
    level: DispatcherLevel,

    turn_left: Aabb2<f32>,
    turn_right: Aabb2<f32>,
}

pub struct DispatcherStateClient {
    active_side: DispatcherViewSide,
}

impl GameDispatcher {
    pub fn new(context: &Context) -> Self {
        const TURN_BUTTON_SIZE: vec2<f32> = vec2(50.0, 50.0);
        Self {
            context: context.clone(),

            final_texture: geng_utils::texture::new_texture(context.geng.ugli(), SCREEN_SIZE),
            framebuffer_size: vec2(1, 1),
            screen: Aabb2::ZERO.extend_positive(vec2(1.0, 1.0)),
            texture_scaling: 1.0,

            cursor_position_raw: vec2::ZERO,
            cursor_position_game: vec2::ZERO,

            client_state: DispatcherStateClient {
                active_side: DispatcherViewSide::Back,
            },
            state: DispatcherState::new(),
            level: context.assets.get().dispatcher.level.clone(),

            turn_left: Aabb2::point(vec2(TURN_BUTTON_SIZE.x / 2.0, SCREEN_SIZE.y as f32 / 2.0))
                .extend_symmetric(TURN_BUTTON_SIZE / 2.0),
            turn_right: Aabb2::point(vec2(
                SCREEN_SIZE.x as f32 - TURN_BUTTON_SIZE.x / 2.0,
                SCREEN_SIZE.y as f32 / 2.0,
            ))
            .extend_symmetric(TURN_BUTTON_SIZE / 2.0),
        }
    }

    fn draw_game(&mut self) {
        let framebuffer = &mut geng_utils::texture::attach_texture(
            &mut self.final_texture,
            self.context.geng.ugli(),
        );
        ugli::clear(
            framebuffer,
            Some(self.context.assets.get().palette.background),
            None,
            None,
        );

        let sprites = &self.context.assets.get().dispatcher.sprites;

        let level = self.level.get_side_mut(self.client_state.active_side);
        for (item, positioning) in &mut level.items {
            let texture = match item {
                DispatcherItem::DoorSign => {
                    if self.state.door_sign_open {
                        &sprites.sign_open
                    } else {
                        &sprites.sign_closed
                    }
                }
                DispatcherItem::Table => &sprites.table,
                DispatcherItem::Monitor => &sprites.monitor,
            };
            let size = positioning
                .size
                .unwrap_or(texture.size().as_f32() * self.texture_scaling);
            let pos = Aabb2::point(positioning.anchor - size * positioning.alignment)
                .extend_positive(size);
            let draw = geng_utils::texture::DrawTexture::new(texture).fit(pos, vec2(0.5, 0.5));
            positioning.hitbox = draw.target;
            draw.draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);
        }

        for (texture, mut target) in [
            (&sprites.arrow_left, self.turn_left),
            (&sprites.arrow_right, self.turn_right),
        ] {
            if target.contains(self.cursor_position_game) {
                target = target.extend_uniform(target.width() * 0.1);
            }
            geng_utils::texture::DrawTexture::new(texture)
                .fit(target, vec2(0.5, 0.5))
                .draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);
        }
    }

    fn cursor_press(&mut self) {
        if self.turn_left.contains(self.cursor_position_game) {
            self.client_state.active_side = self.client_state.active_side.cycle_left();
            return;
        } else if self.turn_right.contains(self.cursor_position_game) {
            self.client_state.active_side = self.client_state.active_side.cycle_right();
            return;
        }

        let level = self.level.get_side_mut(self.client_state.active_side);
        for (item, positioning) in &level.items {
            if positioning.hitbox.contains(self.cursor_position_game) {
                match item {
                    DispatcherItem::DoorSign => {
                        self.state.door_sign_open = !self.state.door_sign_open
                    }
                    _ => {}
                }
            }
        }
    }
}

impl geng::State for GameDispatcher {
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::CursorMove { position } => {
                self.cursor_position_raw = position;
                self.cursor_position_game = (position.as_f32() - self.screen.bottom_left())
                    / self.screen.size()
                    * SCREEN_SIZE.as_f32();
                dbg!(self.cursor_position_game);
            }
            geng::Event::MousePress {
                button: geng::MouseButton::Left,
            } => {
                self.cursor_press();
            }
            _ => (),
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
        self.draw_game();
        let draw = geng_utils::texture::DrawTexture::new(&self.final_texture)
            .fit_screen(vec2(0.5, 0.5), framebuffer);
        self.screen = draw.target;
        draw.draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);
    }
}

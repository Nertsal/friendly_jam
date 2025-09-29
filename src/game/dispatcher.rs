use super::*;

use crate::{assets::*, model::DispatcherState};

use geng_utils::{conversions::Vec2RealConversions, interpolation::SecondOrderState};

const SCREEN_SIZE: vec2<usize> = vec2(1920, 1080);

pub struct GameDispatcher {
    context: Context,

    final_texture: ugli::Texture,
    framebuffer_size: vec2<usize>,
    screen: Aabb2<f32>,
    /// Default scaling from texture to SCREEN_SIZE.
    texture_scaling: f32,
    camera: Camera2d,
    camera_fov: SecondOrderState<f32>,
    camera_center: SecondOrderState<vec2<f32>>,

    cursor_position_raw: vec2<f64>,
    cursor_position_game: vec2<f32>,

    client_state: DispatcherStateClient,
    state: DispatcherState,
    items_layout: HashMap<(DispatcherViewSide, usize), Aabb2<f32>>,
    monitor: Aabb2<f32>,
    monitor_inside: Aabb2<f32>,

    turn_left: Aabb2<f32>,
    turn_right: Aabb2<f32>,
}

pub struct DispatcherStateClient {
    focus: Focus,
    active_side: DispatcherViewSide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Whole,
    Monitor,
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
            camera: Camera2d {
                center: SCREEN_SIZE.as_f32() / 2.0,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Vertical(SCREEN_SIZE.y as f32),
            },
            camera_fov: SecondOrderState::new(1.5, 1.0, 0.0, SCREEN_SIZE.y as f32),
            camera_center: SecondOrderState::new(1.5, 1.0, 0.0, SCREEN_SIZE.as_f32() / 2.0),

            cursor_position_raw: vec2::ZERO,
            cursor_position_game: vec2::ZERO,

            client_state: DispatcherStateClient {
                focus: Focus::Whole,
                active_side: DispatcherViewSide::Back,
            },
            state: DispatcherState::new(),
            items_layout: HashMap::new(),
            monitor: Aabb2::ZERO,
            monitor_inside: Aabb2::ZERO,

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
        let assets = self.context.assets.get();
        let framebuffer = &mut geng_utils::texture::attach_texture(
            &mut self.final_texture,
            self.context.geng.ugli(),
        );
        ugli::clear(framebuffer, Some(assets.palette.background), None, None);

        let sprites = &assets.dispatcher.sprites;

        let level = assets
            .dispatcher
            .level
            .get_side(self.client_state.active_side);
        let mut draw_monitor = false;
        for (item_index, (item, positioning)) in level.items.iter().enumerate() {
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
            let mut draw = geng_utils::texture::DrawTexture::new(texture).fit(pos, vec2(0.5, 0.5));
            if item.is_interactable()
                && draw.target.contains(self.cursor_position_game)
                && !(*item == DispatcherItem::Monitor && self.client_state.focus == Focus::Monitor)
            {
                draw.target = draw.target.extend_uniform(20.0);
            }

            self.items_layout
                .insert((self.client_state.active_side, item_index), draw.target);
            if let DispatcherItem::Monitor = item {
                draw_monitor = true;
                self.monitor = draw.target;
                self.monitor_inside = Aabb2::from_corners(
                    vec2(27.0, -32.0) / vec2(549.0, 513.0) * self.monitor.size(),
                    vec2(519.0, -311.0) / vec2(549.0, 513.0) * self.monitor.size(),
                )
                .translate(self.monitor.top_left());
            }

            draw.draw(&self.camera, &self.context.geng, framebuffer);
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
                .draw(&self.camera, &self.context.geng, framebuffer);
        }

        if draw_monitor {
            // Monitor
            geng_utils::texture::DrawTexture::new(&sprites.workspace)
                .fit(self.monitor_inside, vec2(0.5, 0.5))
                .draw(&self.camera, &self.context.geng, framebuffer);
        }
    }

    fn cursor_press(&mut self) {
        let assets = self.context.assets.get();

        if self.turn_left.contains(self.cursor_position_game) {
            self.client_state.active_side = self.client_state.active_side.cycle_left();
            return;
        } else if self.turn_right.contains(self.cursor_position_game) {
            self.client_state.active_side = self.client_state.active_side.cycle_right();
            return;
        }

        let level = assets
            .dispatcher
            .level
            .get_side(self.client_state.active_side);
        for (item_index, (item, _)) in level.items.iter().enumerate() {
            let Some(&hitbox) = self
                .items_layout
                .get(&(self.client_state.active_side, item_index))
            else {
                continue;
            };
            if hitbox.contains(self.cursor_position_game) {
                match item {
                    DispatcherItem::DoorSign => {
                        self.state.door_sign_open = !self.state.door_sign_open
                    }
                    DispatcherItem::Monitor => {
                        drop(assets);
                        self.change_focus(Focus::Monitor);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    fn change_focus(&mut self, focus: Focus) {
        let (fov, center) = match focus {
            Focus::Whole => (SCREEN_SIZE.y as f32, SCREEN_SIZE.as_f32() / 2.0),
            Focus::Monitor => (
                self.monitor_inside.height() + 50.0,
                self.monitor_inside.center() + vec2(0.0, -20.0),
            ),
        };
        self.camera_fov.target = fov;
        self.camera_center.target = center;
        self.client_state.focus = focus;
    }
}

impl geng::State for GameDispatcher {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.camera_fov.update(delta_time);
        self.camera.fov = Camera2dFov::Vertical(self.camera_fov.current);
        self.camera_center.update(delta_time);
        self.camera.center = self.camera_center.current;
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::CursorMove { position } => {
                self.cursor_position_raw = position;
                let pos = (position.as_f32() - self.screen.bottom_left()) / self.screen.size()
                    * SCREEN_SIZE.as_f32();
                self.cursor_position_game = self.camera.screen_to_world(SCREEN_SIZE.as_f32(), pos);
            }
            geng::Event::MousePress {
                button: geng::MouseButton::Left,
            } => {
                self.cursor_press();
            }
            geng::Event::KeyPress {
                key: geng::Key::Escape,
            } => {
                if let Focus::Monitor = self.client_state.focus {
                    self.change_focus(Focus::Whole);
                }
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

impl DispatcherItem {
    pub fn is_interactable(&self) -> bool {
        match self {
            DispatcherItem::DoorSign => true,
            DispatcherItem::Table => false,
            DispatcherItem::Monitor => true,
        }
    }
}

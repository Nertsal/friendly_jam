use super::*;

use crate::{
    assets::*,
    interop::{ClientConnection, ClientMessage, ServerMessage},
    model::{DispatcherState, SolverState},
    ui::layout::AreaOps,
};

use geng_utils::{conversions::Vec2RealConversions, interpolation::SecondOrderState};

const SCREEN_SIZE: vec2<usize> = vec2(1920, 1080);

pub struct GameDispatcher {
    context: Context,
    connection: ClientConnection,

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
    solver_state: SolverState,
    ui: DispatcherUi,
}

struct DispatcherUi {
    items_layout: HashMap<(DispatcherViewSide, usize), Aabb2<f32>>,
    monitor: Aabb2<f32>,
    monitor_inside: Aabb2<f32>,
    login_code: Vec<Aabb2<f32>>,
    files: Vec<Aabb2<f32>>,
    opened_file: Aabb2<f32>,

    turn_left: Aabb2<f32>,
    turn_right: Aabb2<f32>,
}

pub struct DispatcherStateClient {
    active_side: DispatcherViewSide,
    focus: Focus,
    login_code: Vec<usize>,
    opened_file: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Whole,
    Monitor,
}

impl GameDispatcher {
    pub fn new(context: &Context, connection: ClientConnection) -> Self {
        const TURN_BUTTON_SIZE: vec2<f32> = vec2(50.0, 50.0);
        Self {
            context: context.clone(),
            connection,

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
                active_side: DispatcherViewSide::Back,
                focus: Focus::Whole,
                login_code: vec![],
                opened_file: None,
            },
            state: DispatcherState::new(),
            solver_state: SolverState::new(),
            ui: DispatcherUi {
                items_layout: HashMap::new(),
                monitor: Aabb2::ZERO,
                monitor_inside: Aabb2::ZERO,
                login_code: vec![],
                files: vec![],
                opened_file: Aabb2::ZERO,

                turn_left: Aabb2::point(vec2(TURN_BUTTON_SIZE.x / 2.0, SCREEN_SIZE.y as f32 / 2.0))
                    .extend_symmetric(TURN_BUTTON_SIZE / 2.0),
                turn_right: Aabb2::point(vec2(
                    SCREEN_SIZE.x as f32 - TURN_BUTTON_SIZE.x / 2.0,
                    SCREEN_SIZE.y as f32 / 2.0,
                ))
                .extend_symmetric(TURN_BUTTON_SIZE / 2.0),
            },
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
                DispatcherItem::RealMouse => &sprites.real_mouse,
                DispatcherItem::Cactus => &sprites.cactus,
                DispatcherItem::Book => &sprites.book,
                DispatcherItem::TheSock => &sprites.the_sock,
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

            self.ui
                .items_layout
                .insert((self.client_state.active_side, item_index), draw.target);
            if let DispatcherItem::Monitor = item {
                draw_monitor = true;
                self.ui.monitor = draw.target;
                self.ui.monitor_inside = Aabb2::from_corners(
                    vec2(27.0, -32.0) / vec2(549.0, 513.0) * self.ui.monitor.size(),
                    vec2(519.0, -311.0) / vec2(549.0, 513.0) * self.ui.monitor.size(),
                )
                .translate(self.ui.monitor.top_left())
                .fit_aabb(sprites.login_screen.size().as_f32(), vec2(0.5, 0.5));

                let convert = |pos: vec2<usize>| -> vec2<f32> {
                    pos.as_f32() * vec2(1.0, -1.0) / vec2(1045.0, 685.0)
                        * self.ui.monitor_inside.size()
                };

                let digit = |a: vec2<usize>, b: vec2<usize>| {
                    Aabb2::from_corners(convert(a), convert(b))
                        .translate(self.ui.monitor_inside.top_left())
                };
                self.ui.login_code = vec![
                    digit(vec2(445, 440), vec2(482, 487)),
                    digit(vec2(518, 432), vec2(561, 484)),
                    digit(vec2(586, 435), vec2(618, 482)),
                ];

                let convert = |pos: vec2<usize>| -> vec2<f32> {
                    pos.as_f32() * vec2(1.0, -1.0) / vec2(1039.0, 665.0)
                        * self.ui.monitor_inside.size()
                };

                let file_size = vec2(30.0, 20.0);
                let file = |pos: vec2<usize>| {
                    Aabb2::point(convert(pos))
                        .extend_symmetric(file_size / 2.0)
                        .translate(self.ui.monitor_inside.top_left())
                };
                self.ui.files = vec![
                    file(vec2(135, 465)),
                    file(vec2(205, 465)),
                    file(vec2(275, 465)),
                    file(vec2(345, 465)),
                    file(vec2(415, 465)),
                ];

                self.ui.opened_file =
                    Aabb2::from_corners(convert(vec2(430, 150)), convert(vec2(900, 420)))
                        .translate(self.ui.monitor_inside.top_left());
            }

            draw.draw(&self.camera, &self.context.geng, framebuffer);
        }

        for (texture, mut target) in [
            (&sprites.arrow_left, self.ui.turn_left),
            (&sprites.arrow_right, self.ui.turn_right),
        ] {
            if target.contains(self.cursor_position_game) {
                target = target.extend_uniform(target.width() * 0.1);
            }
            geng_utils::texture::DrawTexture::new(texture)
                .fit(target, vec2(0.5, 0.5))
                .draw(&self.camera, &self.context.geng, framebuffer);
        }

        let monitor_focused = self.client_state.focus == Focus::Monitor;
        if draw_monitor {
            // Monitor
            if self.state.monitor_unlocked {
                // Workspace
                geng_utils::texture::DrawTexture::new(&sprites.workspace)
                    .fit(self.ui.monitor_inside, vec2(0.5, 0.5))
                    .draw(&self.camera, &self.context.geng, framebuffer);

                // Files
                for pos in self
                    .ui
                    .files
                    .iter()
                    .take(self.solver_state.levels_completed + 1)
                {
                    let mut pos = *pos;
                    if monitor_focused && pos.contains(self.cursor_position_game) {
                        pos = pos.extend_uniform(3.0);
                    }
                    geng_utils::texture::DrawTexture::new(&sprites.file)
                        .fit(pos, vec2(0.5, 0.5))
                        .draw(&self.camera, &self.context.geng, framebuffer);
                }

                // Opened file
                if let Some(file) = self.client_state.opened_file {
                    let draw = geng_utils::texture::DrawTexture::new(&sprites.file_window)
                        .fit(self.ui.opened_file, vec2(0.5, 0.5));
                    let window = draw.target;
                    draw.draw(&self.camera, &self.context.geng, framebuffer);

                    if let Some(text) = assets.dispatcher.files.get(file) {
                        draw_text(
                            &assets.font,
                            text,
                            10.0,
                            Rgba::BLACK,
                            window.extend_uniform(-10.0),
                            &self.camera,
                            framebuffer,
                        );
                    }
                }
            } else {
                // Login
                geng_utils::texture::DrawTexture::new(&sprites.login_screen)
                    .fit(self.ui.monitor_inside, vec2(0.5, 0.5))
                    .draw(&self.camera, &self.context.geng, framebuffer);

                // Code
                let font = self.context.geng.default_font();
                for (digit, pos) in self.client_state.login_code.iter().zip(&self.ui.login_code) {
                    self.context.geng.draw2d().draw2d(
                        framebuffer,
                        &self.camera,
                        &draw2d::Text::unit(&**font, digit.to_string(), Rgba::WHITE).fit_into(*pos),
                    );
                }
            }
        }
    }

    fn cursor_press(&mut self) {
        let assets = self.context.assets.get();

        if self.ui.turn_left.contains(self.cursor_position_game) {
            self.client_state.active_side = self.client_state.active_side.cycle_left();
            return;
        } else if self.ui.turn_right.contains(self.cursor_position_game) {
            self.client_state.active_side = self.client_state.active_side.cycle_right();
            return;
        }

        let level = assets
            .dispatcher
            .level
            .get_side(self.client_state.active_side);
        for (item_index, (item, _)) in level.items.iter().enumerate() {
            let Some(&hitbox) = self
                .ui
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

        if let Focus::Monitor = self.client_state.focus {
            if self.client_state.opened_file.is_none()
                && let Some(file) = self
                    .ui
                    .files
                    .iter()
                    .position(|file| file.contains(self.cursor_position_game))
            {
                // Open file
                self.client_state.opened_file = Some(file);
            }
        }
    }

    fn change_focus(&mut self, focus: Focus) {
        if self.client_state.focus == focus {
            return;
        }

        let (fov, center) = match focus {
            Focus::Whole => (SCREEN_SIZE.y as f32, SCREEN_SIZE.as_f32() / 2.0),
            Focus::Monitor => (
                self.ui.monitor_inside.height() + 50.0,
                self.ui.monitor_inside.center() + vec2(0.0, -20.0),
            ),
        };
        self.camera_fov.target = fov;
        self.camera_center.target = center;
        self.client_state.focus = focus;
    }

    fn press_digit(&mut self, digit: usize) {
        if self.client_state.focus == Focus::Monitor
            && !self.state.monitor_unlocked
            && self.client_state.login_code.len() < 3
        {
            self.client_state.login_code.push(digit);
        }
    }

    fn press_escape(&mut self) {
        if let Focus::Monitor = self.client_state.focus {
            if self.client_state.opened_file.take().is_some() {
                return;
            }
            self.change_focus(Focus::Whole);
        }
    }

    fn press_backspace(&mut self) {
        if self.client_state.focus == Focus::Monitor && !self.state.monitor_unlocked {
            self.client_state.login_code.pop();
        }
    }

    fn press_enter(&mut self) {
        if self.client_state.focus == Focus::Monitor && !self.state.monitor_unlocked {
            if self.client_state.login_code == vec![6, 6, 6] {
                self.unlock_monitor();
            } else {
                // TODO
            }
        }
    }

    fn unlock_monitor(&mut self) {
        // TODO: move to a button press
        if !self.state.monitor_unlocked && self.solver_state.levels_completed == 0 {
            self.solver_state.levels_completed += 1;
            self.connection
                .send(ClientMessage::SyncSolverState(self.solver_state.clone()));
        }

        self.state.monitor_unlocked = true;
        self.connection
            .send(ClientMessage::SyncDispatcherState(self.state.clone()));
    }

    fn handle_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Ping | ServerMessage::RoomJoined(..) | ServerMessage::StartGame(..) => {}
            ServerMessage::Error(error) => log::error!("Server error: {error}"),
            ServerMessage::SyncDispatcherState(dispatcher_state) => self.state = dispatcher_state,
            ServerMessage::SyncSolverState(solver_state) => self.solver_state = solver_state,
        }
    }
}

impl geng::State for GameDispatcher {
    fn update(&mut self, delta_time: f64) {
        if let Some(Ok(message)) = self.connection.try_recv() {
            self.handle_message(message);
        }

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
            geng::Event::KeyPress { key } => match key {
                geng::Key::Escape => self.press_escape(),
                geng::Key::Backspace => self.press_backspace(),
                geng::Key::Enter => self.press_enter(),
                geng::Key::Digit0 => self.press_digit(0),
                geng::Key::Digit1 => self.press_digit(1),
                geng::Key::Digit2 => self.press_digit(2),
                geng::Key::Digit3 => self.press_digit(3),
                geng::Key::Digit4 => self.press_digit(4),
                geng::Key::Digit5 => self.press_digit(5),
                geng::Key::Digit6 => self.press_digit(6),
                geng::Key::Digit7 => self.press_digit(7),
                geng::Key::Digit8 => self.press_digit(8),
                geng::Key::Digit9 => self.press_digit(9),
                _ => {}
            },
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
            DispatcherItem::RealMouse => true,
            DispatcherItem::Cactus => true,
            DispatcherItem::Book => true,
            DispatcherItem::TheSock => true,
        }
    }
}

fn draw_text(
    font: &Font,
    text: &str,
    font_size: f32,
    color: Rgba<f32>,
    position: Aabb2<f32>,
    camera: &Camera2d,
    framebuffer: &mut ugli::Framebuffer,
) {
    let lines = crate::util::wrap_text(font, text, position.width() / font_size);
    let row = position.align_aabb(vec2(position.width(), font_size), vec2(0.5, 1.0));
    let rows = row.stack(vec2(0.0, -row.height()), lines.len());

    for (line, position) in lines.into_iter().zip(rows) {
        font.draw(
            framebuffer,
            camera,
            line,
            position.align_pos(vec2(0.0, 0.5)),
            crate::render::util::TextRenderOptions {
                size: font_size,
                color,
                align: vec2(0.0, 0.5),
                ..default()
            },
        );
    }
}

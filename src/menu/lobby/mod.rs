use crate::{
    context::Context,
    interop::{ClientConnection, ClientMessage, RoomInfo, ServerMessage},
    model::GameRole,
    render::{mask::MaskedStack, util::UtilRender},
    ui::{layout::AreaOps, *},
};

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub struct Lobby {
    context: Context,
    ui_context: UiContext,
    ui: LobbyUi,
    mask_stack: MaskedStack,
    util_render: UtilRender,
    transition: Option<geng::state::Transition>,

    state: LobbyState,
}

pub struct LobbyState {
    connection: ClientConnection,
    room_info: RoomInfo,
    selected_role: Option<GameRole>,
}

pub struct LobbyUi {}

impl Lobby {
    pub async fn new(context: &Context, connection: ClientConnection, room_info: RoomInfo) -> Self {
        log::info!("Joined room {}", room_info.code);
        Self {
            context: context.clone(),
            ui_context: UiContext::new(context),
            ui: LobbyUi::new(),
            mask_stack: MaskedStack::new(context),
            util_render: UtilRender::new(context.clone()),
            transition: None,

            state: LobbyState {
                connection,
                room_info,
                selected_role: None,
            },
        }
    }

    fn handle_server_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Ping => self.state.connection.send(ClientMessage::Pong),
            ServerMessage::Error(error) => {
                log::error!("Error: {}", error);
            }
            ServerMessage::RoomJoined(_) => {}
            ServerMessage::StartGame(game_role) => {
                log::info!("Starting game as {:?}", game_role);
                let state: Box<dyn geng::State> = match game_role {
                    GameRole::Dispatcher => Box::new(crate::game::GameDispatcher::new(
                        &self.context,
                        self.state.connection.clone(),
                    )),
                    GameRole::Solver => Box::new(crate::game::GameSolver::new(
                        &self.context,
                        self.state.connection.clone(),
                    )),
                };
                self.transition = Some(geng::state::Transition::Switch(state));
            }
            ServerMessage::SyncDispatcherState(_) | ServerMessage::SyncSolverState(_) => {}
        }
    }
}

impl geng::State for Lobby {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.ui_context.update(delta_time);

        while let Some(message) = self.state.connection.try_recv() {
            if let Ok(message) = message {
                self.handle_server_message(message);
            }
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::CursorMove { position } => {
                self.ui_context.cursor.cursor_move(position.as_f32());
            }
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += delta as f32;
            }
            _ => {}
        }
    }

    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.ui_context.state.frame_start();
        self.ui_context.geometry.update(framebuffer.size());

        self.ui.layout(
            &mut self.state,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            &mut self.ui_context,
        );
        self.ui_context.frame_end();

        ugli::clear(framebuffer, Some(Rgba::BLACK), Some(1.0), None);
        let camera = &geng::PixelPerfectCamera;

        let geometry = RefCell::new(Geometry::new());
        self.ui_context.state.iter_widgets(
            |w| {
                geometry.borrow_mut().merge(w.draw_top(&self.ui_context));
            },
            |w| {
                geometry.borrow_mut().merge(w.draw(&self.ui_context));
            },
        );
        let geometry = geometry.into_inner();

        self.util_render
            .draw_geometry(&mut self.mask_stack, geometry, camera, framebuffer);
    }
}

impl LobbyState {
    pub fn select_role(&mut self, role: GameRole) {
        self.selected_role = Some(role);
        self.connection.send(ClientMessage::SelectRole(role));
    }
}

impl LobbyUi {
    pub fn new() -> Self {
        Self {}
    }

    pub fn layout(&mut self, state: &mut LobbyState, screen: Aabb2<f32>, context: &mut UiContext) {
        context.screen = screen;
        context.font_size = screen.height() * 0.05;
        context.layout_size = screen.height() * 0.07;
        let atlas = &context.context.assets.get().atlas;

        let mut code = screen;
        let mut solver = code.split_bottom(0.66);
        let dispatcher = solver.split_right(0.5);

        let code_text = context.state.get_root_or(|| TextWidget::new(""));
        code_text.text = format!("Код комнаты: {}", state.room_info.code).into();
        code_text.update(code, context);

        let button = context
            .state
            .get_root_or(|| ButtonWidget::new(atlas.button_background()).with_text("Диспетчер"));
        button.update(dispatcher, context);
        if button.state.mouse_left.clicked {
            state.select_role(GameRole::Dispatcher);
        }

        let button = context
            .state
            .get_root_or(|| ButtonWidget::new(atlas.button_background()).with_text("Беглец"));
        button.update(solver, context);
        if button.state.mouse_left.clicked {
            state.select_role(GameRole::Solver);
        }
    }
}

use crate::{
    context::Context,
    interop::{ClientMessage, ServerMessage},
    menu::loading_screen::LoadingScreen,
    render::{mask::MaskedStack, util::UtilRender},
    ui::{layout::AreaOps, *},
};

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub struct MainMenu {
    context: Context,
    ui_context: UiContext,
    ui: MainMenuUi,
    mask_stack: MaskedStack,
    util_render: UtilRender,

    connect: Option<String>,
    state: MainMenuState,
}

pub struct MainMenuState {
    action: Option<Action>,
}

enum Action {
    CreateRoom,
    Join(String),
}

pub struct MainMenuUi {}

impl MainMenu {
    pub async fn new(context: &Context, connect: Option<String>) -> Self {
        Self {
            context: context.clone(),
            ui_context: UiContext::new(context),
            ui: MainMenuUi::new(),
            mask_stack: MaskedStack::new(&context.geng, &context.assets),
            util_render: UtilRender::new(context.clone()),

            connect,
            state: MainMenuState { action: None },
        }
    }
}

impl geng::State for MainMenu {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.ui_context.update(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::CursorMove { position } => {
                self.ui_context.cursor.cursor_move(position.as_f32());
            }
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += delta as f32;
            }
            geng::Event::EditText(text) => {
                if self.ui_context.text_edit.any_active() {
                    self.ui_context.text_edit.set_text(text);
                }
            }
            _ => (),
        }
    }

    fn transition(&mut self) -> Option<geng::state::Transition> {
        if let Some(action) = self.state.action.take() {
            let state: Box<dyn geng::State> = match action {
                Action::CreateRoom => {
                    log::info!("Creating a room...");
                    let context = self.context.clone();
                    let connect = self.connect.clone();
                    let future = async move {
                        let mut connection =
                            geng::net::client::connect(&connect.unwrap()).await.unwrap();
                        connection.send(ClientMessage::CreateRoom);
                        let room_info = loop {
                            let message = connection.next().await.unwrap().unwrap();
                            match message {
                                ServerMessage::Ping => connection.send(ClientMessage::Pong),
                                ServerMessage::RoomJoined(room_info) => break room_info,
                                _ => {
                                    log::error!("Failed to create the room");
                                    return None;
                                }
                            }
                        };
                        Some(crate::menu::lobby::Lobby::new(&context, connection, room_info).await)
                    }
                    .boxed_local();
                    Box::new(LoadingScreen::new(
                        &self.context.geng,
                        geng::EmptyLoadingScreen::new(&self.context.geng),
                        future,
                    ))
                }
                Action::Join(code) => {
                    log::info!("Joining room {code}...");
                    let context = self.context.clone();
                    let connect = self.connect.clone();
                    let future = async move {
                        let mut connection =
                            geng::net::client::connect(&connect.unwrap()).await.unwrap();
                        connection.send(ClientMessage::JoinRoom(code));
                        let room_info = loop {
                            let message = connection.next().await.unwrap().unwrap();
                            match message {
                                ServerMessage::Ping => connection.send(ClientMessage::Pong),
                                ServerMessage::RoomJoined(room_info) => break room_info,
                                _ => {
                                    log::error!("Failed to join the room");
                                    return None;
                                }
                            }
                        };
                        Some(crate::menu::lobby::Lobby::new(&context, connection, room_info).await)
                    }
                    .boxed_local();
                    Box::new(LoadingScreen::new(
                        &self.context.geng,
                        geng::EmptyLoadingScreen::new(&self.context.geng),
                        future,
                    ))
                }
            };
            return Some(geng::state::Transition::Push(state));
        }

        None
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

        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);

        let camera = &geng::PixelPerfectCamera;
        ugli::clear(framebuffer, Some(Rgba::TRANSPARENT_BLACK), Some(1.0), None);

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

impl MainMenuUi {
    pub fn new() -> Self {
        Self {}
    }

    pub fn layout(
        &mut self,
        state: &mut MainMenuState,
        screen: Aabb2<f32>,
        context: &mut UiContext,
    ) {
        context.screen = screen;
        context.font_size = screen.height() * 0.05;
        context.layout_size = screen.height() * 0.07;
        let atlas = &context.context.assets.atlas;

        let mut create = screen;
        let mut join = create.split_bottom(0.5);
        let code = join.split_right(0.5);

        let button = context
            .state
            .get_root_or(|| ButtonWidget::new(atlas.button_background()).with_text("Create room"));
        button.update(create, context);
        if button.state.mouse_left.clicked {
            state.action = Some(Action::CreateRoom);
        }

        let join_button = context
            .state
            .get_root_or(|| ButtonWidget::new(atlas.button_background()).with_text("Join"));
        join_button.update(join, context);

        let code_input = context.state.get_root_or(|| InputWidget::new(""));
        code_input.update(code, context);

        if join_button.state.mouse_left.clicked {
            state.action = Some(Action::Join(code_input.raw.clone()));
        }
    }
}

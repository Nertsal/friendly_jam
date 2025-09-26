use crate::{
    context::Context,
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

    state: MainMenuState,
}

pub struct MainMenuState {
    selected_role: Option<Role>,
}

pub enum Role {
    Dispatcher,
    Solver,
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

            state: MainMenuState {
                selected_role: None,
            },
        }
    }
}

impl geng::State for MainMenu {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.ui_context.update(delta_time);
    }

    fn transition(&mut self) -> Option<geng::state::Transition> {
        if let Some(role) = self.state.selected_role.take() {
            let state: Box<dyn geng::State> = match role {
                Role::Dispatcher => Box::new(crate::game::GameDispatcher::new(&self.context)),
                Role::Solver => Box::new(crate::game::GameSolver::new(&self.context)),
            };
            return Some(geng::state::Transition::Switch(state));
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
        let atlas = &context.context.assets.atlas;

        let mut solver = screen;
        let dispatcher = solver.split_left(0.5);

        let button = context
            .state
            .get_root_or(|| ButtonWidget::new(atlas.play_dispatcher()));
        button.layout(dispatcher, context);
        if button.state.mouse_left.clicked {
            state.selected_role = Some(Role::Dispatcher);
        }

        let button = context
            .state
            .get_root_or(|| ButtonWidget::new(atlas.play_solver()));
        button.layout(solver, context);
        if button.state.mouse_left.clicked {
            state.selected_role = Some(Role::Solver);
        }
    }
}

use super::*;

use crate::model::*;

use geng_utils::conversions::*;

const SCREEN_SIZE: vec2<usize> = vec2(1920, 1080);

pub struct GameSolver {
    context: Context,

    final_texture: ugli::Texture,
    framebuffer_size: vec2<usize>,
    screen: Aabb2<f32>,
    /// Default scaling from texture to SCREEN_SIZE.
    texture_scaling: f32,

    cursor_position_raw: vec2<f64>,
    cursor_position_game: vec2<f32>,

    client_state: SolverStateClient,
    state: SolverState,
    camera: Camera2d,

    player_control: PlayerControl,
}

struct SolverStateClient {
    player: Player,
}

struct PlayerControl {
    pub jump: bool,
    pub hold_jump: bool,
    pub move_dir: vec2<FCoord>,
}

impl PlayerControl {
    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

impl Default for PlayerControl {
    fn default() -> Self {
        Self {
            jump: false,
            hold_jump: false,
            move_dir: vec2::ZERO,
        }
    }
}

struct Player {
    pub collider: Collider,
    pub velocity: vec2<FCoord>,
    pub state: PlayerState,
    pub control_timeout: Option<FTime>,
    pub facing_left: bool,
    pub can_hold_jump: bool,
    pub coyote_time: Option<FTime>,
    pub jump_buffer: Option<FTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum PlayerState {
    Grounded,
    Airborn,
}

impl GameSolver {
    pub fn new(context: &Context) -> Self {
        Self {
            context: context.clone(),

            final_texture: geng_utils::texture::new_texture(context.geng.ugli(), SCREEN_SIZE),
            framebuffer_size: vec2(1, 1),
            screen: Aabb2::ZERO.extend_positive(vec2(1.0, 1.0)),
            texture_scaling: 1.0,

            cursor_position_raw: vec2::ZERO,
            cursor_position_game: vec2::ZERO,

            client_state: SolverStateClient {
                player: Player {
                    collider: Collider::aabb(
                        Aabb2::point(vec2(0.0, 0.0))
                            .extend_positive(vec2(1.0, 1.0))
                            .as_r32(),
                    ),
                    velocity: vec2::ZERO,
                    state: PlayerState::Airborn,
                    control_timeout: None,
                    facing_left: false,
                    can_hold_jump: false,
                    coyote_time: None,
                    jump_buffer: None,
                },
            },
            state: SolverState::new(),
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Vertical(10.0),
            },

            player_control: PlayerControl::default(),
        }
    }

    fn draw_game(&mut self) {
        let framebuffer = &mut geng_utils::texture::attach_texture(
            &mut self.final_texture,
            self.context.geng.ugli(),
        );
        ugli::clear(
            framebuffer,
            Some(self.context.assets.palette.background),
            None,
            None,
        );

        self.context.geng.draw2d().quad(
            framebuffer,
            &self.camera,
            self.client_state.player.collider.compute_aabb().as_f32(),
            Rgba::RED,
        );
    }

    fn update_player(&mut self, delta_time: FTime) {
        let state = &mut self.client_state;
        let rules = &self.context.assets.solver.rules;
        state.player.update_timers(delta_time);

        // Update Jump Buffer
        if self.player_control.jump {
            state.player.jump_buffer = Some(rules.buffer_time);
        }

        // Update Jump Hold
        if state.player.can_hold_jump && !self.player_control.hold_jump {
            state.player.can_hold_jump = false;
        }

        // Update look direction
        let player = &mut state.player;
        if player.facing_left && player.velocity.x > FCoord::ZERO
            || !player.facing_left && player.velocity.x < FCoord::ZERO
        {
            player.facing_left = !player.facing_left;
        }

        // Apply gravity
        state.player.velocity += rules.gravity * delta_time;

        self.player_variable_jump(delta_time);
        self.player_horizontal_control(delta_time);
        self.player_jump(delta_time);

        self.player_move(delta_time);
        self.player_update_state();

        self.player_control.take();
    }

    fn player_variable_jump(&mut self, delta_time: FTime) {
        let state = &mut self.client_state;
        let rules = &self.context.assets.solver.rules;

        // Variable jump height
        if state.player.velocity.y < FCoord::ZERO {
            // Faster drop
            state.player.velocity.y +=
                rules.gravity.y * (rules.fall_multiplier - FCoord::ONE) * delta_time;
            let cap = rules.free_fall_speed;
            state.player.velocity.y = state.player.velocity.y.clamp_abs(cap);
        } else if state.player.velocity.y > FCoord::ZERO
            && !(self.player_control.hold_jump && state.player.can_hold_jump)
        {
            // Low jump
            state.player.velocity.y +=
                rules.gravity.y * (rules.low_multiplier - FCoord::ONE) * delta_time;
        }
    }

    fn player_horizontal_control(&mut self, delta_time: FTime) {
        let state = &mut self.client_state;
        let rules = &self.context.assets.solver.rules;

        if state.player.control_timeout.is_some() {
            return;
        }

        // Horizontal speed control
        let current = state.player.velocity.x;
        let max_speed = rules.move_speed;
        let target = self.player_control.move_dir.x * max_speed;

        let mut acc = FCoord::ZERO;

        // Acceleration
        let is_grounded = matches!(state.player.state, PlayerState::Grounded);
        if target == FCoord::ZERO
            || target.signum() != current.signum()
            || target.abs() > current.abs()
        {
            // Accelerate towards target
            acc += if is_grounded {
                rules.acceleration_ground
            } else {
                rules.acceleration_air
            };
        } else {
            // Target is aligned with current velocity and is higher
            // Decelerate
            acc += if is_grounded {
                rules.deceleration_ground
            } else {
                rules.deceleration_air
            };
        }

        state.player.velocity.x += (target - current).clamp_abs(acc * delta_time);
    }

    fn player_jump(&mut self, _delta_time: FTime) {
        let state = &mut self.client_state;
        let rules = &self.context.assets.solver.rules;

        if state.player.jump_buffer.is_none() {
            return;
        }

        // Try jump
        let jump = match state.player.state {
            PlayerState::Grounded => true,
            PlayerState::Airborn => state.player.coyote_time.is_some(),
        };
        if !jump {
            return;
        }

        // Use jump
        state.player.coyote_time = None;
        state.player.jump_buffer = None;
        state.player.can_hold_jump = true;
        let player = &mut state.player;
        let push = if self.player_control.move_dir.x == FCoord::ZERO {
            FCoord::ZERO
        } else {
            rules.jump_push * self.player_control.move_dir.x.signum()
        };
        let jump_vel = vec2(player.velocity.x + push, rules.jump_strength);
        player.velocity = jump_vel;
        player.state = PlayerState::Airborn;
        // self.world.assets.sounds.jump.play();
        // self.spawn_particles(ParticleSpawn {
        //     lifetime: Time::ONE,
        //     position: actor.collider.feet(),
        //     velocity: vec2(Coord::ZERO, Coord::ONE),
        //     amount: 3,
        //     color: Rgba::WHITE,
        //     radius: Coord::new(0.1),
        //     ..Default::default()
        // });
    }

    fn player_check_ground(&mut self) {
        let rules = &self.context.assets.solver.rules;
        let player = &mut self.client_state.player;
        let was_grounded = matches!(player.state, PlayerState::Grounded);
        if was_grounded {
            player.state = PlayerState::Airborn;
        }
        let update_state = (matches!(player.state, PlayerState::Airborn) || was_grounded)
            && player.velocity.y <= FCoord::ZERO;

        if update_state {
            let collider = player.feet_collider();

            if self.check_collision(&collider).is_some() {
                let player = &mut self.client_state.player;
                player.state = PlayerState::Grounded;
                player.coyote_time = Some(rules.coyote_time);

                // if !was_grounded {
                //     // Just landed
                //     let spawn = ParticleSpawn {
                //         lifetime: Time::ONE,
                //         position: actor.collider.feet(),
                //         velocity: vec2(Coord::ZERO, Coord::ONE) * Coord::new(0.5),
                //         amount: 3,
                //         color: Rgba::WHITE,
                //         radius: Coord::new(0.1),
                //         ..Default::default()
                //     };
                //     self.spawn_particles(spawn);
                // }
            }
        }
    }

    fn player_move(&mut self, delta_time: FTime) {
        let player = &mut self.client_state.player;
        player.collider.position += player.velocity * delta_time;

        if let Some(collision) = self.check_collision(&self.client_state.player.collider) {
            let player = &mut self.client_state.player;
            player.collider.position -= collision.normal * collision.penetration;
            player.velocity -= collision.normal * vec2::dot(player.velocity, collision.normal);
        }
    }

    fn player_update_state(&mut self) {
        self.player_check_ground();
    }

    fn check_collision(&self, collider: &Collider) -> Option<Collision> {
        let floor = Collider::aabb(
            Aabb2::ZERO
                .extend_symmetric(vec2(50.0, 0.0).as_r32())
                .extend_down(r32(1.0)),
        );
        if let Some(col) = collider.collide(&floor) {
            return Some(col);
        }

        // TODO: walls and stuff
        None
    }
}

impl geng::State for GameSolver {
    fn update(&mut self, delta_time: f64) {
        let delta_time = FTime::new(delta_time as f32);

        let window = self.context.geng.window();
        let controls = &self.context.assets.solver.controls;
        if geng_utils::key::is_key_pressed(window, &controls.move_left) {
            self.player_control.move_dir += vec2(-1.0, 0.0).as_r32();
        }
        if geng_utils::key::is_key_pressed(window, &controls.move_right) {
            self.player_control.move_dir += vec2(1.0, 0.0).as_r32();
        }
        if geng_utils::key::is_key_pressed(window, &controls.jump) {
            self.player_control.hold_jump = true;
        }

        self.update_player(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        let controls = &self.context.assets.solver.controls;
        if geng_utils::key::is_event_press(&event, &controls.jump) {
            self.player_control.jump = true;
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

impl Player {
    fn update_timers(&mut self, delta_time: FTime) {
        // Coyote Time
        if let Some(time) = &mut self.coyote_time {
            *time -= delta_time;
            if *time <= FTime::ZERO {
                self.coyote_time = None;
            }
        }

        // Jump Buffer
        if let Some(time) = &mut self.jump_buffer {
            *time -= delta_time;
            if *time <= FTime::ZERO {
                self.jump_buffer = None;
            }
        }

        // Controll timeout
        if let Some(time) = &mut self.control_timeout {
            // No horizontal control
            *time -= delta_time;
            if *time <= FTime::ZERO {
                self.control_timeout = None;
            }
        }
    }

    fn feet_collider(&self) -> Collider {
        let aabb = self.collider.compute_aabb();
        Collider::aabb(aabb.extend_up(-aabb.height() * r32(0.8)))
    }
}

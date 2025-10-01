use super::*;

use crate::{
    assets::SolverItem,
    interop::{ClientConnection, ClientMessage, ServerMessage},
    model::*,
};

use geng_utils::conversions::*;

const SCREEN_SIZE: vec2<usize> = vec2(1920, 1080);
const LEVEL_SIZE: vec2<f32> = vec2(16.0, 9.0);

pub struct GameSolver {
    context: Context,
    connection: ClientConnection,

    final_texture: ugli::Texture,
    framebuffer_size: vec2<usize>,
    screen: Aabb2<f32>,
    /// Default scaling from texture to SCREEN_SIZE.
    texture_scaling: f32,

    cursor_position_raw: vec2<f64>,
    cursor_position_game: vec2<f32>,

    client_state: SolverStateClient,
    state: SolverState,
    dispatcher_state: DispatcherState,
    camera: Camera2d,

    player_control: PlayerControl,
}

struct SolverStateClient {
    player: Player,
    level_static_colliders: Vec<Collider>,
    door_entrance: Collider,
    door_exit: Collider,
    platforms: Vec<Collider>,
    items: Vec<SolverItem>,
    picked_up_item: Option<SolverItem>,
    explosion: Option<(vec2<FCoord>, FTime)>,
}

struct PlayerControl {
    pub jump: bool,
    pub hold_jump: bool,
    pub move_dir: vec2<FCoord>,
    pub pickup: bool,
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
            pickup: false,
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
    pub animation_time: FTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum PlayerState {
    Grounded,
    Airborn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PlayerAnimationState {
    Idle,
    Running,
    Jumping,
}

impl GameSolver {
    pub fn new(context: &Context, connection: ClientConnection, test: Option<usize>) -> Self {
        let assets = context.assets.get();
        assets.sounds.music.play();
        let mut game = Self {
            context: context.clone(),
            connection,

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
                            .extend_positive(vec2(1.0, 1.5))
                            .as_r32(),
                    ),
                    velocity: vec2::ZERO,
                    state: PlayerState::Airborn,
                    control_timeout: None,
                    facing_left: false,
                    can_hold_jump: false,
                    coyote_time: None,
                    jump_buffer: None,
                    animation_time: FTime::ZERO,
                },
                level_static_colliders: Vec::new(),
                door_entrance: Collider::aabb(Aabb2::ZERO),
                door_exit: Collider::aabb(Aabb2::ZERO),
                platforms: Vec::new(),
                items: Vec::new(),
                picked_up_item: None,
                explosion: None,
            },
            state: SolverState::new(),
            dispatcher_state: DispatcherState::new(),
            camera: Camera2d {
                center: LEVEL_SIZE / 2.0,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Cover {
                    width: LEVEL_SIZE.x,
                    height: LEVEL_SIZE.y,
                    scale: 1.0,
                },
            },

            player_control: PlayerControl::default(),
        };

        if let Some(test) = test {
            game.state.current_level = test;
            game.state.levels_completed = test;
        }

        game.reload_level();
        game
    }

    fn reload_level(&mut self) {
        self.player_respawn();
        self.update_level_colliders();
        self.reset_items();
    }

    fn reset_items(&mut self) {
        let assets = self.context.assets.get();
        let Some(level) = assets.solver.levels.get(self.state.current_level) else {
            return;
        };
        self.client_state.items = level.items.clone();
        self.client_state.picked_up_item = None;
    }

    fn draw_game(&mut self) {
        let framebuffer = &mut geng_utils::texture::attach_texture(
            &mut self.final_texture,
            self.context.geng.ugli(),
        );
        let assets = self.context.assets.get();
        ugli::clear(framebuffer, Some(assets.palette.background), None, None);

        // Background
        if self.state.current_level == 0 {
            geng_utils::texture::DrawTexture::new(&assets.solver.sprites.level1)
                .fit(Aabb2::ZERO.extend_positive(LEVEL_SIZE), vec2(0.5, 0.5))
                .draw(&self.camera, &self.context.geng, framebuffer);
        }

        // Bounds
        geng_utils::texture::DrawTexture::new(&assets.solver.sprites.level_bounds)
            .fit(Aabb2::ZERO.extend_positive(LEVEL_SIZE), vec2(0.5, 0.5))
            .draw(&self.camera, &self.context.geng, framebuffer);

        // Doors
        geng_utils::texture::DrawTexture::new(&assets.solver.sprites.door_closed)
            .transformed(mat3::scale(vec2(-1.0, 1.0)))
            .fit_height(self.client_state.door_entrance.compute_aabb().as_f32(), 0.0)
            .draw(&self.camera, &self.context.geng, framebuffer);
        geng_utils::texture::DrawTexture::new(if self.state.is_exit_open() {
            &assets.solver.sprites.door_open
        } else {
            &assets.solver.sprites.door_closed
        })
        .fit_height(self.client_state.door_exit.compute_aabb().as_f32(), 1.0)
        .draw(&self.camera, &self.context.geng, framebuffer);

        // Platforms
        for platform in &self.client_state.platforms {
            geng_utils::texture::DrawTexture::new(&assets.solver.sprites.platform)
                .fit_width(platform.compute_aabb().as_f32(), 1.0)
                .draw(&self.camera, &self.context.geng, framebuffer);
        }

        // Items
        for item in &self.client_state.items {
            let texture = assets.solver.sprites.item_texture(item.kind);
            geng_utils::texture::DrawTexture::new(texture)
                .fit(item.collider.compute_aabb().as_f32(), vec2(0.5, 0.5))
                .draw(&self.camera, &self.context.geng, framebuffer);
        }

        // Player
        let player = &self.client_state.player;
        let animation = |frames: &[Rc<crate::assets::PixelTexture>], frame_time: f32| {
            let frame_time = r32(frame_time);
            let frame = (player.animation_time / frame_time)
                .as_f32()
                .max(0.0)
                .floor() as usize
                % frames.len();
            frames[frame].clone()
        };
        let texture = match player.animation_state() {
            PlayerAnimationState::Idle => animation(&assets.solver.sprites.player.idle, 0.5),
            PlayerAnimationState::Running => animation(&assets.solver.sprites.player.running, 0.1),
            PlayerAnimationState::Jumping => {
                let sprites = &assets.solver.sprites.player.jump;
                if player.velocity.y > FCoord::ZERO {
                    sprites[0].clone()
                } else {
                    sprites[1].clone()
                }
            }
        };
        let flip = !player.facing_left;
        geng_utils::texture::DrawTexture::new(&texture)
            .transformed(mat3::scale(vec2(if flip { -1.0 } else { 1.0 }, 1.0)))
            .fit_width(player.collider.compute_aabb().as_f32(), 0.0)
            .draw(&self.camera, &self.context.geng, framebuffer);

        // Held item
        if let Some(item) = &self.client_state.picked_up_item {
            let texture = assets.solver.sprites.item_texture(item.kind);
            let mut collider = item.collider.clone();
            let dir = if self.client_state.player.facing_left {
                vec2(-1.0, 0.0)
            } else {
                vec2(1.0, 0.0)
            }
            .as_r32();
            collider.position = self.client_state.player.collider.position + dir * r32(0.5);
            geng_utils::texture::DrawTexture::new(texture)
                .fit(collider.compute_aabb().as_f32(), vec2(0.5, 0.5))
                .draw(&self.camera, &self.context.geng, framebuffer);
        }

        // Explosion
        if let Some((pos, time)) = self.client_state.explosion {
            let frames = &assets.solver.sprites.explosion;
            let frame = (time.as_f32() * frames.len() as f32).floor() as usize;
            if let Some(frame) = frames.get(frame) {
                geng_utils::texture::DrawTexture::new(&frame.texture)
                    .fit(
                        Aabb2::point(pos.as_f32()).extend_uniform(1.0),
                        vec2(0.5, 0.5),
                    )
                    .draw(&self.camera, &self.context.geng, framebuffer);
            }
        }
    }

    fn player_respawn(&mut self) {
        self.client_state.level_static_colliders.clear();
        let assets = self.context.assets.get();
        let Some(level) = assets.solver.levels.get(self.state.current_level) else {
            return;
        };

        let player = &mut self.client_state.player;
        player.collider.position =
            level.spawnpoint + vec2(r32(0.0), player.collider.compute_aabb().height() / r32(2.0));
        player.velocity = vec2::ZERO;
    }

    fn update_level_colliders(&mut self) {
        self.client_state.level_static_colliders.clear();
        let assets = self.context.assets.get();
        let Some(level) = assets.solver.levels.get(self.state.current_level) else {
            return;
        };

        let wall_thickness = r32(1.0);
        let door_height = r32(2.0);

        // Floor
        self.client_state
            .level_static_colliders
            .push(Collider::aabb(
                Aabb2::ZERO
                    .extend_right(r32(LEVEL_SIZE.x))
                    .extend_up(wall_thickness),
            ));
        // Left wall
        self.client_state
            .level_static_colliders
            .push(Collider::aabb(
                Aabb2::point(vec2(r32(0.0), door_height + wall_thickness))
                    .extend_up(r32(LEVEL_SIZE.y))
                    .extend_right(wall_thickness),
            ));
        // Right wall
        self.client_state
            .level_static_colliders
            .push(Collider::aabb(
                Aabb2::point(vec2(LEVEL_SIZE.x.as_r32(), door_height + wall_thickness))
                    .extend_up(r32(LEVEL_SIZE.y))
                    .extend_left(wall_thickness),
            ));
        // Ceiling
        self.client_state
            .level_static_colliders
            .push(Collider::aabb(
                Aabb2::point(vec2(0.0, LEVEL_SIZE.y).as_r32())
                    .extend_right(r32(LEVEL_SIZE.x))
                    .extend_down(wall_thickness),
            ));

        let door_width = r32(0.3);

        // Entrance door
        self.client_state.door_entrance = Collider::aabb(
            Aabb2::point(vec2(0.0.as_r32(), wall_thickness))
                .extend_up(door_height)
                .extend_right(door_width),
        );

        // Exit door
        self.client_state.door_exit = Collider::aabb(
            Aabb2::point(vec2(LEVEL_SIZE.x.as_r32(), wall_thickness))
                .extend_up(door_height)
                .extend_left(door_width),
        );

        // Platforms
        let platform_size = assets.solver.sprites.platform.size().as_f32();
        self.client_state.platforms = level
            .platforms
            .iter()
            .map(|platform| {
                let size = vec2(
                    platform.width,
                    platform.width / platform_size.aspect().as_r32(),
                );
                Collider::aabb(
                    Aabb2::point(platform.pos)
                        .extend_symmetric(vec2(size.x, r32(0.0) / r32(2.0)))
                        .extend_down(size.y),
                )
            })
            .collect();
    }

    fn update_items(&mut self, delta_time: FTime) {
        // Item movement
        for item in &mut self.client_state.items {
            if item.has_gravity {
                let collision = self
                    .client_state
                    .level_static_colliders
                    .iter()
                    .chain(&self.client_state.platforms)
                    .filter_map(|static_col| item.collider.collide(static_col))
                    .max_by_key(|col| col.penetration);
                match collision {
                    None => {
                        item.collider.position += vec2(0.0, -5.0).as_r32() * delta_time;
                    }
                    Some(collision) => {
                        item.collider.position -= collision.normal * collision.penetration;
                    }
                }
            }
        }

        // Item collision
        let items_count = self.client_state.items.len();
        let mut remove_items = Vec::new();
        for i in 0..items_count {
            for j in i + 1..items_count {
                if let Ok([item, other]) = self.client_state.items.get_disjoint_mut([i, j]) {
                    let check_combination = |a, b| {
                        item.kind == a && other.kind == b || item.kind == b && other.kind == a
                    };

                    use crate::assets::SolverItemKind::*;
                    if check_combination(Fish, CinderBlock) && item.collider.check(&other.collider)
                    {
                        // Explosion
                        remove_items.extend([i, j]);
                        self.client_state.explosion = Some((item.collider.position, FTime::ZERO));
                    }
                }
            }
        }
        remove_items.sort();
        for i in remove_items.into_iter().rev() {
            self.client_state.items.swap_remove(i);
        }
    }

    fn update_player(&mut self, delta_time: FTime) {
        let anim_state = self.client_state.player.animation_state();

        {
            let state = &mut self.client_state;
            let rules = &self.context.assets.get().solver.rules;
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
        }

        if self.player_control.pickup {
            if let Some(mut item) = self.client_state.picked_up_item.take() {
                // Drop item
                let dir = if self.client_state.player.facing_left {
                    vec2(-1.0, 0.0)
                } else {
                    vec2(1.0, 0.0)
                }
                .as_r32();
                item.collider.position =
                    self.client_state.player.collider.position + dir * r32(0.5);
                self.client_state.items.push(item);
            } else if let Some(i) = self.client_state.items.iter().position(|item| {
                item.can_pickup && item.collider.check(&self.client_state.player.collider)
            }) {
                // Pick up an item
                self.client_state.picked_up_item = Some(self.client_state.items.swap_remove(i));
            }
        }

        self.player_variable_jump(delta_time);
        self.player_horizontal_control(delta_time);
        self.player_jump(delta_time);

        self.player_move(delta_time);
        self.player_update_state();

        if anim_state != self.client_state.player.animation_state() {
            self.client_state.player.animation_time = FTime::ZERO;
        }

        self.check_transition();
        self.check_out_of_bounds();

        self.player_control.take();
    }

    fn player_variable_jump(&mut self, delta_time: FTime) {
        let state = &mut self.client_state;
        let rules = &self.context.assets.get().solver.rules;

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
        let rules = &self.context.assets.get().solver.rules;

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
        let rules = &self.context.assets.get().solver.rules;

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
        let rules = &self.context.assets.get().solver.rules;
        let player = &mut self.client_state.player;
        let was_grounded = matches!(player.state, PlayerState::Grounded);
        if was_grounded {
            player.state = PlayerState::Airborn;
        }
        let update_state = (matches!(player.state, PlayerState::Airborn) || was_grounded)
            && player.velocity.y <= FCoord::ZERO;

        if update_state {
            let collider = player.feet_collider();

            if self.check_ground_collision(&collider).is_some() {
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

        let fix_collision = |player: &mut Player, collision: &Collision| {
            player.collider.position -= collision.normal * collision.penetration;
            player.velocity -= collision.normal * vec2::dot(player.velocity, collision.normal);
        };
        let collide_with = |player: &mut Player, other: &Collider| {
            if let Some(collision) = player.collider.collide(other) {
                fix_collision(player, &collision);
            }
        };

        // Static colliders
        for static_col in &self.client_state.level_static_colliders {
            collide_with(player, static_col);
        }

        // Doors
        collide_with(player, &self.client_state.door_entrance);
        if !self.state.is_exit_open() {
            collide_with(player, &self.client_state.door_exit);
        }

        // Platforms
        if player.velocity.y.as_f32() <= 0.0 {
            let collider = player.feet_collider();
            for platform in &self.client_state.platforms {
                if let Some(collision) = collider.collide(platform) {
                    fix_collision(player, &collision);
                }
            }
        }

        // Items
        for item in &mut self.client_state.items {
            if item.pushable
                && let Some(collision) = player.collider.collide(&item.collider)
            {
                let offset = collision.normal * collision.penetration;
                // let velocity_offset =
                //     collision.normal * vec2::dot(player.velocity, collision.normal);
                player.collider.position -= offset * r32(0.5);
                // player.velocity -= velocity_offset * r32(0.5);
                item.collider.position += vec2::UNIT_X * vec2::dot(vec2::UNIT_X, offset) * r32(0.5);
            }
        }
    }

    fn player_update_state(&mut self) {
        self.player_check_ground();
    }

    fn check_ground_collision(&self, collider: &Collider) -> Option<Collision> {
        self.client_state
            .level_static_colliders
            .iter()
            .chain(&self.client_state.platforms)
            .filter_map(|static_col| collider.collide(static_col))
            .max_by_key(|col| col.penetration)
    }

    fn check_transition(&mut self) {
        let assets = self.context.assets.get();
        let Some(level) = assets.solver.levels.get(self.state.current_level) else {
            return;
        };
        let player = &self.client_state.player;
        if self.state.is_exit_open() && player.collider.check(&Collider::aabb(level.transition)) {
            self.state.current_level += 1;
            self.connection
                .send(ClientMessage::SyncSolverState(self.state.clone()));
            drop(assets);
            self.player_respawn();
            self.update_level_colliders();
        }
    }

    fn check_out_of_bounds(&mut self) {
        let player = &self.client_state.player;
        if player.collider.position.y < r32(-50.0) {
            self.player_respawn();
        }
    }

    fn handle_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Ping | ServerMessage::RoomJoined(..) | ServerMessage::StartGame(..) => {}
            ServerMessage::Error(error) => log::error!("Server error: {error}"),
            ServerMessage::SyncDispatcherState(dispatcher_state) => {
                self.dispatcher_state = dispatcher_state
            }
            ServerMessage::SyncSolverState(solver_state) => self.state = solver_state,
        }
    }
}

impl geng::State for GameSolver {
    fn update(&mut self, delta_time: f64) {
        if let Some(Ok(message)) = self.connection.try_recv() {
            self.handle_message(message);
        }

        let delta_time = FTime::new(delta_time as f32);

        {
            let window = self.context.geng.window();
            let controls = &self.context.assets.get().solver.controls;
            if geng_utils::key::is_key_pressed(window, &controls.move_left) {
                self.player_control.move_dir += vec2(-1.0, 0.0).as_r32();
            }
            if geng_utils::key::is_key_pressed(window, &controls.move_right) {
                self.player_control.move_dir += vec2(1.0, 0.0).as_r32();
            }
            if geng_utils::key::is_key_pressed(window, &controls.jump) {
                self.player_control.hold_jump = true;
            }
        }

        if let Some((pos, timer)) = &mut self.client_state.explosion {
            *timer += delta_time;
            if timer.as_f32() > 1.0 {
                if (self.client_state.player.collider.position - *pos)
                    .len()
                    .as_f32()
                    < 1.5
                {
                    panic!("ты взорвался");
                }

                self.client_state.explosion = None;
                if self.state.current_level == 1 && !self.state.is_exit_open() {
                    self.state.levels_completed += 1;
                    self.connection
                        .send(ClientMessage::SyncSolverState(self.state.clone()));
                }
            }
        }

        self.update_player(delta_time);
        self.update_items(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        let assets = self.context.assets.get();
        let controls = &assets.solver.controls;
        if geng_utils::key::is_event_press(&event, &controls.jump) {
            self.player_control.jump = true;
        }

        if geng_utils::key::is_event_press(&event, &controls.pickup) {
            self.player_control.pickup = true;
        }

        if let geng::Event::KeyPress { key: geng::Key::F5 } = event {
            drop(assets);
            self.reload_level();
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
        self.animation_time += delta_time;

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

        // Control timeout
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
        Collider::aabb(
            aabb.extend_symmetric(-vec2(aabb.width() * r32(0.05), r32(0.0)))
                .extend_up(-aabb.height() * r32(0.8)),
        )
    }

    fn animation_state(&self) -> PlayerAnimationState {
        match self.state {
            PlayerState::Grounded => {
                if self.velocity.x.abs() > r32(0.01) {
                    PlayerAnimationState::Running
                } else {
                    PlayerAnimationState::Idle
                }
            }
            PlayerState::Airborn => PlayerAnimationState::Jumping,
        }
    }
}

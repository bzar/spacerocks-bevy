use bevy::{asset::LoadState, prelude::*};

#[derive(Copy, Clone, Debug)]
enum AsteroidSize {
    Tiny = 0,
    Small,
    Medium,
    Large,
}

impl AsteroidSize {
    fn smaller(&self) -> Option<AsteroidSize> {
        match self {
            AsteroidSize::Tiny => None,
            AsteroidSize::Small => Some(AsteroidSize::Tiny),
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Large => Some(AsteroidSize::Medium),
        }
    }
}

const ASTEROID_SIZES: usize = 4;
const ASTEROID_VARIANTS: usize = 12;

enum ShipWeapon {
    Rapid,
    Spread,
    Beam,
    Plasma,
}

#[derive(Component, Clone, Copy)]
enum ShipProjectile {
    Rapid,
    Spread,
    Beam { power: f32 },
    Plasma { power: f32 },
}

#[derive(Component)]
struct Beam {
    length: f32,
}

#[derive(Clone, Copy)]
enum ShipTurn {
    Neutral,
    Left,
    Right,
}

impl Default for ShipTurn {
    fn default() -> Self {
        Self::Neutral
    }
}

#[derive(Component)]
struct Expiring {
    life: f32,
}

impl Default for ShipWeapon {
    fn default() -> Self {
        Self::Rapid
    }
}

#[derive(Component)]
struct Asteroid {
    size: AsteroidSize,
    integrity: i32,
    variant: usize,
}

#[derive(Component)]
struct Ufo {
    start_position: Vec2,
    end_position: Vec2,
    frequency: f32,
    amplitude: f32,
    duration: f32,
    time: f32,
    shoot_delay: f32,
    shoot_accuracy: f32,
    life: i32,
}
#[derive(Component)]
struct UfoLaser;

#[derive(Component)]
enum Powerup {
    Laser = 0,
    Spread,
    Beam,
    Plasma,
    ExtraLife,
    LoseLife,
    Shield,
}

#[derive(Component, Default)]
struct Moving {
    velocity: Vec2,
    acceleration: Vec2,
}

#[derive(Component)]
struct Spinning {
    speed: f32,
}

#[derive(Component)]
struct Wrapping;

#[derive(Component)]
struct LevelEntity;

#[derive(Component, Default)]
struct Ship {
    throttle: bool,
    turn: ShipTurn,
    fire: bool,
    weapon: ShipWeapon,
    weapon_rapid_level: u8,
    weapon_spread_level: u8,
    weapon_beam_level: u8,
    weapon_plasma_level: u8,
    weapon_cooldown: f32,
    shield_level: u8,
    lives: u8,
}

#[derive(Default)]
struct ShipImages {
    rapid: Handle<Image>,
    rapid_accelerating: Handle<Image>,
    rapid_left: Handle<Image>,
    rapid_left_accelerating: Handle<Image>,
    rapid_right: Handle<Image>,
    rapid_right_accelerating: Handle<Image>,
    spread: Handle<Image>,
    spread_accelerating: Handle<Image>,
    spread_left: Handle<Image>,
    spread_left_accelerating: Handle<Image>,
    spread_right: Handle<Image>,
    spread_right_accelerating: Handle<Image>,
    beam: Handle<Image>,
    beam_accelerating: Handle<Image>,
    beam_left: Handle<Image>,
    beam_left_accelerating: Handle<Image>,
    beam_right: Handle<Image>,
    beam_right_accelerating: Handle<Image>,
    plasma: Handle<Image>,
    plasma_accelerating: Handle<Image>,
    plasma_left: Handle<Image>,
    plasma_left_accelerating: Handle<Image>,
    plasma_right: Handle<Image>,
    plasma_right_accelerating: Handle<Image>,
}

#[derive(Default)]
struct UfoImages {
    ship: Vec<Handle<Image>>,
    laser: Handle<Image>,
}

#[derive(Default)]
struct PowerupImages {
    laser: Handle<Image>,
    spread: Handle<Image>,
    beam: Handle<Image>,
    plasma: Handle<Image>,
    extra_life: Handle<Image>,
    lose_life: Handle<Image>,
    shield: Handle<Image>,
}

#[derive(Default, Resource)]
struct SpriteSheets {
    asteroids: Handle<TextureAtlas>,
    images: Vec<HandleUntyped>,
    ship: ShipImages,
    ufo: UfoImages,
    powerup: PowerupImages,
}

#[derive(Resource)]
struct GameState {
    level: u32,
    score: u32,
    next_ufo_score: u32,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    LoadLevel,
    InGame,
    #[default]
    Loading,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SpriteSheets::default())
        .insert_resource(GameState {
            level: 0,
            score: 0,
            next_ufo_score: random_ufo_interval(),
        })
        .add_system(init.on_startup())
        .add_state::<AppState>()
        .add_system(loading.in_set(OnUpdate(AppState::Loading)))
        .add_system(load_level.in_schedule(OnEnter(AppState::LoadLevel)))
        .add_systems(
            (
                ship_control_system,
                ship_physics,
                ship_sprite,
                moving_system,
                spinning_system,
                wrapping_system,
                expiring_system,
                ship_projectile_asteroid_hit_system,
                asteroid_split_system,
                level_finished_system,
                ufo_spawn_system,
                ufo_movement_system,
                ufo_animation_system,
                ufo_shoot_system,
                ship_projectile_ufo_hit_system,
            )
                .in_set(OnUpdate(AppState::InGame)),
        )
        .add_systems((ship_powerup_collision_system,).in_set(OnUpdate(AppState::InGame)))
        .add_system(despawn_tagged::<LevelEntity>.in_schedule(OnExit(AppState::InGame)))
        .run();
}

fn despawn_tagged<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn asteroid_texture_index(variant: usize, size: AsteroidSize) -> usize {
    variant * ASTEROID_SIZES + size as usize
}

fn asteroid_sprite_rects() -> impl Iterator<Item = Rect> {
    let variant_rows = 5;
    let variant_sizes = [8, 16, 32, 48];
    let variant_width: u32 = variant_sizes.iter().sum();
    let variant_height = variant_sizes.into_iter().max().unwrap_or(0);

    (0..ASTEROID_VARIANTS).flat_map(move |variant_index| {
        let variant_x = (variant_index as u32 / variant_rows) * variant_width;
        let variant_y = (variant_index as u32 % variant_rows) * variant_height;

        variant_sizes
            .into_iter()
            .scan(0, |size_x, size| {
                let result = (size_x.clone(), size);
                *size_x += size;
                Some(result)
            })
            .map(move |(size_x, size)| Rect {
                min: Vec2::new((variant_x + size_x) as f32, variant_y as f32),
                max: Vec2::new(
                    (variant_x + size_x + size - 1) as f32,
                    (variant_y + size - 1) as f32,
                ),
            })
    })
}
fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sprite_sheets: ResMut<SpriteSheets>,
) {
    sprite_sheets.images = asset_server.load_folder("img").unwrap();
    commands.spawn(Camera2dBundle::default());
}

fn loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sprite_sheets: ResMut<SpriteSheets>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut loading_text: Local<Option<Entity>>,
) {
    if loading_text.is_none() {
        *loading_text = Some(
            commands
                .spawn(TextBundle {
                    text: Text::from_section(
                        "Loading...",
                        TextStyle {
                            font: asset_server.load("fonts/DejaVuSans.ttf"),
                            font_size: 100.0,
                            color: Color::WHITE,
                        },
                    ),
                    style: Style {
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .id(),
        );
    }

    let handles = sprite_sheets.images.iter().map(|h| h.id());
    if let LoadState::Loaded = asset_server.get_group_load_state(handles) {
        // Initialize texture atlases
        let asteroid_texture = asset_server.load("img/asteroids.png");
        let mut asteroid_atlas = TextureAtlas::new_empty(asteroid_texture, Vec2::new(512.0, 256.0));

        for asteroid_rect in asteroid_sprite_rects() {
            asteroid_atlas.add_texture(asteroid_rect);
        }

        sprite_sheets.asteroids = texture_atlases.add(asteroid_atlas);

        sprite_sheets.ship = ShipImages {
            rapid: asset_server.load("img/ship-rapid.png"),
            rapid_accelerating: asset_server.load("img/ship-rapid_accelerating.png"),
            rapid_left: asset_server.load("img/ship-rapid_left.png"),
            rapid_left_accelerating: asset_server.load("img/ship-rapid_left_accelerating.png"),
            rapid_right: asset_server.load("img/ship-rapid_right.png"),
            rapid_right_accelerating: asset_server.load("img/ship-rapid_right_accelerating.png"),
            spread: asset_server.load("img/ship-spread.png"),
            spread_accelerating: asset_server.load("img/ship-spread_accelerating.png"),
            spread_left: asset_server.load("img/ship-spread_left.png"),
            spread_left_accelerating: asset_server.load("img/ship-spread_left_accelerating.png"),
            spread_right: asset_server.load("img/ship-spread_right.png"),
            spread_right_accelerating: asset_server.load("img/ship-spread_right_accelerating.png"),
            beam: asset_server.load("img/ship-beam.png"),
            beam_accelerating: asset_server.load("img/ship-beam_accelerating.png"),
            beam_left: asset_server.load("img/ship-beam_left.png"),
            beam_left_accelerating: asset_server.load("img/ship-beam_left_accelerating.png"),
            beam_right: asset_server.load("img/ship-beam_right.png"),
            beam_right_accelerating: asset_server.load("img/ship-beam_right_accelerating.png"),
            plasma: asset_server.load("img/ship-plasma.png"),
            plasma_accelerating: asset_server.load("img/ship-plasma_accelerating.png"),
            plasma_left: asset_server.load("img/ship-plasma_left.png"),
            plasma_left_accelerating: asset_server.load("img/ship-plasma_left_accelerating.png"),
            plasma_right: asset_server.load("img/ship-plasma_right.png"),
            plasma_right_accelerating: asset_server.load("img/ship-plasma_right_accelerating.png"),
        };

        sprite_sheets.ufo = UfoImages {
            ship: vec![
                asset_server.load("img/ufo_1.png"),
                asset_server.load("img/ufo_2.png"),
                asset_server.load("img/ufo_3.png"),
                asset_server.load("img/ufo_4.png"),
            ],
            laser: asset_server.load("img/ufolaser.png"),
        };

        sprite_sheets.powerup = PowerupImages {
            laser: asset_server.load("img/powerup_laser.png"),
            spread: asset_server.load("img/powerup_spread.png"),
            beam: asset_server.load("img/powerup_beam.png"),
            plasma: asset_server.load("img/powerup_plasma.png"),
            extra_life: asset_server.load("img/powerup_extralife.png"),
            lose_life: asset_server.load("img/powerup_loselife.png"),
            shield: asset_server.load("img/powerup_shield.png"),
        };
        // Loading finished
        if let Some(entity) = *loading_text {
            commands.entity(entity).despawn();
        }
        next_state.set(AppState::LoadLevel);
    }
}

fn level_asteroids(level: u32) -> impl Iterator<Item = AsteroidSize> {
    let cost = |size| match size {
        AsteroidSize::Tiny => 1,
        AsteroidSize::Small => 2 + 2 * 1,
        AsteroidSize::Medium => 3 + 2 * 2 * 4 * 1,
        AsteroidSize::Large => 4 + 2 * 3 + 4 * 2 * 8 * 1,
    };

    let budget = (level % 20 + 2) * cost(AsteroidSize::Large);
    let sizes: &[AsteroidSize] = match level {
        0..=4 => &[AsteroidSize::Large],
        5..=8 => &[AsteroidSize::Large, AsteroidSize::Medium],
        9..=12 => &[
            AsteroidSize::Large,
            AsteroidSize::Medium,
            AsteroidSize::Small,
        ],
        _ => &[
            AsteroidSize::Large,
            AsteroidSize::Medium,
            AsteroidSize::Small,
            AsteroidSize::Tiny,
        ],
    };

    sizes.iter().cycle().scan(budget, move |budget, &size| {
        if *budget >= cost(size) {
            *budget -= cost(size);
            Some(size)
        } else if *budget > 0 {
            *budget -= 1;
            Some(AsteroidSize::Tiny)
        } else {
            None
        }
    })
}
fn load_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sprite_sheets: Res<SpriteSheets>,
    game_state: Res<GameState>,
    mut app_state: ResMut<NextState<AppState>>,
    mut ships_query: Query<&mut Transform, With<Ship>>,
) {
    println!("setup level {}", game_state.level);

    let asteroid_variant = game_state.level as usize % ASTEROID_VARIANTS;

    let background_texture =
        asset_server.load(&format!("img/background-{}.png", game_state.level % 11 + 1));
    commands
        .spawn(SpriteBundle {
            texture: background_texture,
            transform: Transform::from_xyz(0.0, 0.0, -0.01),
            ..Default::default()
        })
        .insert(LevelEntity);

    for size in level_asteroids(game_state.level) {
        let distance: f32 = 100.0 * (rand::random::<f32>() + 1.0);
        let pos: Vec2 =
            Vec2::from_angle(rand::random::<f32>() * 2.0 * std::f32::consts::TAU) * distance;
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: sprite_sheets.asteroids.clone(),
                sprite: TextureAtlasSprite::new(asteroid_texture_index(asteroid_variant, size)),
                transform: Transform::from_translation(pos.extend(0.)),
                ..Default::default()
            })
            .insert(Moving {
                velocity: Vec2::new(
                    (rand::random::<f32>() - 0.5) * 10.0,
                    (rand::random::<f32>() - 0.5) * 10.0,
                ),
                ..Default::default()
            })
            .insert(Spinning { speed: 0.2 })
            .insert(Wrapping)
            .insert(Asteroid {
                size,
                integrity: size as i32 * 4 + 1,
                variant: asteroid_variant,
            })
            .insert(LevelEntity);
    }

    if ships_query.is_empty() {
        let ship = Ship {
            weapon_rapid_level: 4,
            weapon_spread_level: 4,
            weapon_plasma_level: 8,
            ..Ship::default()
        };
        commands
            .spawn(SpriteBundle {
                texture: sprite_sheets.ship.choose(&ship),
                ..Default::default()
            })
            .insert(Moving::default())
            .insert(Wrapping)
            .insert(ship);
    } else {
        for mut transform in ships_query.iter_mut() {
            transform.translation = Vec3::ZERO;
        }
    }

    app_state.set(AppState::InGame);
}
fn moving_system(mut moving_query: Query<(&mut Moving, &mut Transform)>, time: Res<Time>) {
    for (mut moving, mut transform) in moving_query.iter_mut() {
        let velocity_delta = moving.acceleration * time.delta().as_secs_f32();
        moving.velocity += velocity_delta;
        transform.translation += (moving.velocity * time.delta().as_secs_f32()).extend(0.0);
    }
}
fn spinning_system(mut spinning_query: Query<(&Spinning, &mut Transform)>, time: Res<Time>) {
    for (spinning, mut transform) in spinning_query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(spinning.speed * time.delta().as_secs_f32());
    }
}

fn expiring_system(
    mut commands: Commands,
    mut expiring_query: Query<(Entity, &mut Expiring)>,
    time: Res<Time>,
) {
    for (entity, mut expiring) in expiring_query.iter_mut() {
        expiring.life -= time.delta().as_secs_f32();
        if expiring.life < 0.0 {
            commands.entity(entity).despawn_recursive()
        }
    }
}

fn wrapping_system(mut wrapping_query: Query<&mut Transform, With<Wrapping>>) {
    for mut transform in wrapping_query.iter_mut() {
        if transform.translation.x > 400.0 {
            transform.translation.x -= 800.0;
        } else if transform.translation.x < -400.0 {
            transform.translation.x += 800.0;
        }

        if transform.translation.y > 240.0 {
            transform.translation.y -= 480.0;
        } else if transform.translation.y < -240.0 {
            transform.translation.y += 480.0;
        }
    }
}
fn ship_control_system(mut ship_query: Query<&mut Ship>, keyboard_input: Res<Input<KeyCode>>) {
    let throttle = keyboard_input.pressed(KeyCode::W);
    let turn_left = keyboard_input.pressed(KeyCode::A);
    let turn_right = keyboard_input.pressed(KeyCode::D);
    let fire = keyboard_input.pressed(KeyCode::E);
    let weapon_rapid = keyboard_input.pressed(KeyCode::Key1);
    let weapon_spread = keyboard_input.pressed(KeyCode::Key2);
    let weapon_beam = keyboard_input.pressed(KeyCode::Key3);
    let weapon_plasma = keyboard_input.pressed(KeyCode::Key4);

    for mut ship in ship_query.iter_mut() {
        ship.throttle = throttle;
        ship.turn = match (turn_left, turn_right) {
            (true, false) => ShipTurn::Left,
            (false, true) => ShipTurn::Right,
            _ => ShipTurn::Neutral,
        };
        ship.fire = fire;
        if weapon_rapid {
            ship.weapon = ShipWeapon::Rapid;
        } else if weapon_spread {
            ship.weapon = ShipWeapon::Spread;
        } else if weapon_beam {
            ship.weapon = ShipWeapon::Beam;
        } else if weapon_plasma {
            ship.weapon = ShipWeapon::Plasma;
        }
    }
}

fn spawn_projectile(
    commands: &mut Commands,
    projectile: ShipProjectile,
    texture: Handle<Image>,
    velocity: Vec2,
    transform: Transform,
    life: f32,
) {
    commands
        .spawn(SpriteBundle {
            texture,
            transform,
            ..Default::default()
        })
        .insert(Moving {
            velocity,
            ..Default::default()
        })
        .insert(Wrapping)
        .insert(projectile)
        .insert(Expiring { life });
}
fn ship_physics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ship_query: Query<(Entity, &mut Ship, &mut Moving, &mut Transform)>,
    mut beam_query: Query<(&mut Beam, &mut Transform), Without<Ship>>,
    time: Res<Time>,
) {
    let time_delta = time.delta().as_secs_f32();

    for (ship_entity, mut ship, mut moving, mut transform) in ship_query.iter_mut() {
        let angular_velocity = match ship.turn {
            ShipTurn::Neutral => 0.0,
            ShipTurn::Left => 3.0,
            ShipTurn::Right => -3.0,
        };
        let acceleration = if ship.throttle { 50.0 } else { 0.0 };
        transform.rotation *= Quat::from_rotation_z(angular_velocity * time_delta);
        moving.acceleration = (transform.rotation * Vec3::Y * acceleration).truncate();

        if ship.weapon_cooldown > 0.0 {
            ship.weapon_cooldown -= time_delta;
        }

        if ship.fire && ship.weapon_cooldown <= 0.0 {
            let projectile = match ship.weapon {
                ShipWeapon::Rapid => ShipProjectile::Rapid,
                ShipWeapon::Spread => ShipProjectile::Spread,
                ShipWeapon::Beam => ShipProjectile::Beam { power: 20.0 },
                ShipWeapon::Plasma => ShipProjectile::Plasma {
                    power: lerp(4.0, 20.0, (ship.weapon_plasma_level - 1) as f32 / 8.0),
                },
            };
            let texture_path = match ship.weapon {
                ShipWeapon::Rapid => "img/laser.png",
                ShipWeapon::Spread => "img/shot.png",
                ShipWeapon::Beam => "img/continuous_tip.png",
                ShipWeapon::Plasma => "img/plasma.png",
            };
            let texture = asset_server.load(texture_path);

            match ship.weapon {
                ShipWeapon::Rapid => {
                    let left_turret = transform.translation
                        + transform.rotation * Quat::from_rotation_z(1.55) * Vec3::Y * 8.0;
                    let right_turret = transform.translation
                        + transform.rotation * Quat::from_rotation_z(-1.55) * Vec3::Y * 8.0;
                    let velocity = (transform.rotation * Vec3::Y * 1200.0).truncate();
                    let left_transform = Transform {
                        translation: left_turret,
                        rotation: transform.rotation.clone(),
                        ..Default::default()
                    };
                    let right_transform = Transform {
                        translation: right_turret,
                        rotation: transform.rotation.clone(),
                        ..Default::default()
                    };
                    spawn_projectile(
                        &mut commands,
                        projectile,
                        texture.clone(),
                        velocity.clone(),
                        left_transform,
                        0.25,
                    );
                    spawn_projectile(
                        &mut commands,
                        projectile,
                        texture,
                        velocity,
                        right_transform,
                        0.25,
                    );
                    ship.weapon_cooldown =
                        lerp(0.3, 0.05, (ship.weapon_rapid_level - 1) as f32 / 8.0);
                }
                ShipWeapon::Spread => {
                    let spread_angle =
                        lerp(0.314, 3.0, (ship.weapon_spread_level - 1) as f32 / 8.0);
                    let shots = 2 * ship.weapon_spread_level + 1;
                    for i in 0..shots {
                        let rotation = transform.rotation
                            * Quat::from_rotation_z(
                                spread_angle * i as f32 / (shots - 1) as f32 - spread_angle / 2.0,
                            );
                        let velocity = (rotation * Vec3::Y).truncate() * 1200.0;
                        let transform = Transform {
                            translation: transform.translation,
                            ..Default::default()
                        };
                        spawn_projectile(
                            &mut commands,
                            projectile,
                            texture.clone(),
                            velocity,
                            transform,
                            0.20,
                        );
                    }
                    ship.weapon_cooldown =
                        lerp(0.8, 0.3, (ship.weapon_spread_level - 1) as f32 / 8.0);
                }
                ShipWeapon::Plasma => {
                    let power = lerp(4.0, 20.0, (ship.weapon_plasma_level - 1) as f32 / 8.0);
                    let velocity = (transform.rotation * Vec3::Y * 1000.0).truncate();
                    let translation = transform.translation.clone();
                    let rotation = Quat::from_rotation_z(1.57) * transform.rotation;
                    let scale = Vec3::splat(power / 16.0);
                    let transform = Transform {
                        translation,
                        rotation,
                        scale,
                    };
                    spawn_projectile(&mut commands, projectile, texture, velocity, transform, 0.5);
                    ship.weapon_cooldown =
                        lerp(1.2, 0.8, (ship.weapon_plasma_level - 1) as f32 / 8.0);
                }
                ShipWeapon::Beam => {
                    if beam_query.is_empty() {
                        let length = 16.0;
                        let transform = Transform::from_xyz(0.0, 64.0, 0.0);
                        let tip = commands
                            .spawn(SpriteBundle {
                                texture,
                                transform,
                                ..Default::default()
                            })
                            .insert(projectile)
                            .id();
                        let texture = asset_server.load("img/continuous_beam.png");
                        let mut transform = Transform::from_xyz(0.0, length / 2.0, 0.0);
                        transform.scale.y = length / 128.0;
                        let beam = commands
                            .spawn(SpriteBundle {
                                texture,
                                transform,
                                ..Default::default()
                            })
                            .insert(Beam { length })
                            .insert(projectile)
                            .id();
                        commands.entity(beam).push_children(&[tip]);
                        commands.entity(ship_entity).push_children(&[beam]);
                    } else {
                        for (mut beam, mut transform) in beam_query.iter_mut() {
                            beam.length += time_delta * 32.0;
                            transform.translation.y = beam.length / 2.0;
                            transform.scale.y = beam.length / 128.0;
                        }
                    }
                }
            };
        }
    }
}

fn lerp(start: f32, end: f32, position: f32) -> f32 {
    start + (end - start) * position
}

impl ShipImages {
    fn choose(&self, ship: &Ship) -> Handle<Image> {
        use {ShipTurn::*, ShipWeapon::*};
        match (&ship.weapon, ship.turn, ship.throttle) {
            (Rapid, Neutral, false) => &self.rapid,
            (Rapid, Neutral, true) => &self.rapid_accelerating,
            (Rapid, Left, false) => &self.rapid_left,
            (Rapid, Left, true) => &self.rapid_left_accelerating,
            (Rapid, Right, false) => &self.rapid_right,
            (Rapid, Right, true) => &self.rapid_right_accelerating,
            (Spread, Neutral, false) => &self.spread,
            (Spread, Neutral, true) => &self.spread_accelerating,
            (Spread, Left, false) => &self.spread_left,
            (Spread, Left, true) => &self.spread_left_accelerating,
            (Spread, Right, false) => &self.spread_right,
            (Spread, Right, true) => &self.spread_right_accelerating,
            (Beam, Neutral, false) => &self.beam,
            (Beam, Neutral, true) => &self.beam_accelerating,
            (Beam, Left, false) => &self.beam_left,
            (Beam, Left, true) => &self.beam_left_accelerating,
            (Beam, Right, false) => &self.beam_right,
            (Beam, Right, true) => &self.beam_right_accelerating,
            (Plasma, Neutral, false) => &self.plasma,
            (Plasma, Neutral, true) => &self.plasma_accelerating,
            (Plasma, Left, false) => &self.plasma_left,
            (Plasma, Left, true) => &self.plasma_left_accelerating,
            (Plasma, Right, false) => &self.plasma_right,
            (Plasma, Right, true) => &self.plasma_right_accelerating,
        }
        .clone_weak()
    }
}

fn ship_sprite(
    mut ship_query: Query<(&Ship, &mut Handle<Image>)>,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (ship, mut image) in ship_query.iter_mut() {
        *image = sprite_sheets.ship.choose(&ship);
    }
}

fn ship_projectile_asteroid_hit_system(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut ShipProjectile, &mut Transform)>,
    mut asteroids: Query<(&mut Asteroid, &Transform), Without<ShipProjectile>>,
) {
    for (projectile_entity, projectile, mut projectile_transform) in projectiles.iter_mut() {
        for (mut asteroid, asteroid_transform) in asteroids.iter_mut() {
            let asteroid_radius: f32 = match asteroid.size {
                AsteroidSize::Tiny => 4.0,
                AsteroidSize::Small => 8.0,
                AsteroidSize::Medium => 16.0,
                AsteroidSize::Large => 24.0,
            };
            match *projectile {
                ShipProjectile::Rapid | ShipProjectile::Spread => {
                    if projectile_transform
                        .translation
                        .distance_squared(asteroid_transform.translation)
                        < asteroid_radius.powi(2)
                    {
                        commands.entity(projectile_entity).despawn();
                        if asteroid.integrity > 0 {
                            asteroid.integrity -= 1;
                        }
                    }
                }
                ShipProjectile::Plasma { mut power } => {
                    if projectile_transform
                        .translation
                        .distance_squared(asteroid_transform.translation)
                        < (asteroid_radius + power).powi(2)
                    {
                        let distance = projectile_transform
                            .translation
                            .distance(asteroid_transform.translation);
                        let effect =
                            (asteroid_radius + power - distance).min(asteroid.integrity as f32);
                        power -= effect;
                        if power <= 0.0 {
                            commands.entity(projectile_entity).despawn();
                        } else {
                            projectile_transform.scale = Vec3::splat(power / 16.0);
                        }
                        if asteroid.integrity > 0 {
                            asteroid.integrity -= effect.ceil() as i32;
                        }
                    }
                }
                ShipProjectile::Beam { .. } => {}
            }
        }
    }
}

fn asteroid_split_system(
    mut commands: Commands,
    asteroids: Query<(Entity, &Asteroid, &Transform)>,
    sprite_sheets: Res<SpriteSheets>,
    mut game_state: ResMut<GameState>,
) {
    for (asteroid_entity, asteroid, transform) in asteroids.iter() {
        if asteroid.integrity <= 0 {
            game_state.score += asteroid_score(asteroid.size);
            commands.entity(asteroid_entity).despawn();
            if let Some(size) = asteroid.size.smaller() {
                let direction = (transform.rotation * transform.translation)
                    .truncate()
                    .normalize();
                let data = [direction, -direction];
                let transform = transform.clone();

                for dir in data {
                    commands
                        .spawn(SpriteSheetBundle {
                            texture_atlas: sprite_sheets.asteroids.clone(),
                            sprite: TextureAtlasSprite {
                                index: asteroid_texture_index(asteroid.variant, size),
                                ..Default::default()
                            },
                            transform,
                            ..Default::default()
                        })
                        .insert(Moving {
                            velocity: dir * 30.0,
                            ..Default::default()
                        })
                        .insert(Spinning { speed: 0.2 })
                        .insert(Wrapping)
                        .insert(Asteroid {
                            size,
                            integrity: size as i32 * 4 + 1,
                            variant: asteroid.variant,
                        });
                }
            }
        }
    }
}

fn level_finished_system(
    asteroids_query: Query<Entity, With<Asteroid>>,
    mut game_state: ResMut<GameState>,
    mut state: ResMut<NextState<AppState>>,
) {
    if asteroids_query.is_empty() {
        game_state.level += 1;
        state.set(AppState::LoadLevel);
    }
}

fn ufo_spawn_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    sprite_sheets: Res<SpriteSheets>,
) {
    if game_state.score >= game_state.next_ufo_score {
        game_state.next_ufo_score += random_ufo_interval();
        let horizontal: bool = rand::random();
        let direction: bool = rand::random();
        let span = if horizontal { 800.0 } else { 480.0 };
        let d = rand::random::<f32>() * span;
        let position = Vec2::new(
            if !horizontal {
                d
            } else if direction {
                0.
            } else {
                800.
            },
            if horizontal {
                d
            } else if direction {
                0.
            } else {
                480.
            },
        ) - Vec2::new(400.0, 240.0);

        let start_position = position;
        let end_position = -position;
        let frequency = rand::random::<f32>() * 5.0;
        let amplitude = rand::random::<f32>() * 90.0 + 10.0;
        let duration = 20.0 - 10.0 * (game_state.level as f32 / 40.0).min(1.0);
        let time = 0.0;
        let shoot_delay = 2.0; // FIXME
        let shoot_accuracy = 0.75; // FIXME
        let transform = Transform::from_translation(start_position.extend(0.));
        let texture = sprite_sheets.ufo.ship[0].clone();
        let life = 20;
        commands
            .spawn(SpriteBundle {
                texture,
                transform,
                ..Default::default()
            })
            .insert(Ufo {
                start_position,
                end_position,
                frequency,
                amplitude,
                duration,
                time,
                shoot_delay,
                shoot_accuracy,
                life,
            })
            .insert(LevelEntity);
    }
}

fn ufo_movement_system(
    mut commands: Commands,
    mut ufos_query: Query<(Entity, &mut Ufo, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut ufo, mut transform) in ufos_query.iter_mut() {
        ufo.time += time.delta_seconds();
        let t = ufo.time / ufo.duration;
        let journey = ufo.end_position - ufo.start_position;
        let deviation = ufo.amplitude * f32::sin(ufo.frequency * std::f32::consts::TAU * t);
        let position = ufo.start_position + journey * t + journey.normalize().perp() * deviation;
        let angle = 10.0 * std::f32::consts::TAU * t;
        let rotation = Quat::from_rotation_z(angle);
        *transform = Transform::from_rotation(rotation).with_translation(position.extend(0.));

        if ufo.time >= ufo.duration {
            commands.entity(entity).despawn();
        }
    }
}
fn ufo_animation_system(
    mut ufos_query: Query<(&Ufo, &mut Handle<Image>)>,
    sprite_sheets: Res<SpriteSheets>,
) {
    let frame_duration = 1. / 5.;
    for (ufo, mut image) in ufos_query.iter_mut() {
        let frame = (ufo.time / frame_duration) as usize % sprite_sheets.ufo.ship.len();
        *image = sprite_sheets.ufo.ship[frame].clone();
    }
}
fn ufo_shoot_system(
    mut commands: Commands,
    mut ufos_query: Query<(&mut Ufo, &Transform)>,
    ships_query: Query<&Transform, With<Ship>>,
    sprite_sheets: Res<SpriteSheets>,
    time: Res<Time>,
) {
    let ship_transform = ships_query.single();
    for (mut ufo, ufo_transform) in ufos_query.iter_mut() {
        ufo.shoot_delay -= time.delta_seconds();
        if ufo.shoot_delay <= 0.0 {
            ufo.shoot_delay = 2.0; // FIXME
            let target = (ship_transform.translation - ufo_transform.translation)
                .truncate()
                .normalize();
            let aim_error =
                (1.0 - ufo.shoot_accuracy) * (rand::random::<f32>() - 0.5) * std::f32::consts::PI;
            let aim = Vec2::from_angle(aim_error).rotate(target);
            let speed = 500.0; // FIXME
            let velocity = aim * speed;
            let angle = Vec2::Y.angle_between(aim);
            let transform = Transform::from_translation(ufo_transform.translation)
                .with_rotation(Quat::from_rotation_z(angle));
            let texture = sprite_sheets.ufo.laser.clone();
            let life = 2.0;
            commands
                .spawn(SpriteBundle {
                    texture,
                    transform,
                    ..Default::default()
                })
                .insert(UfoLaser)
                .insert(Moving {
                    velocity,
                    acceleration: Vec2::ZERO,
                })
                .insert(Expiring { life });
        }
    }
}
fn random_ufo_interval() -> u32 {
    const MIN: f32 = 400.0;
    const MAX: f32 = 800.0;
    (rand::random::<f32>() * (MAX - MIN) + MIN) as u32
}
fn asteroid_score(size: AsteroidSize) -> u32 {
    match size {
        AsteroidSize::Tiny => 50,
        AsteroidSize::Small => 100,
        AsteroidSize::Medium => 150,
        AsteroidSize::Large => 200,
    }
}
fn ship_projectile_ufo_hit_system(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut ShipProjectile, &mut Transform)>,
    mut ufos: Query<(Entity, &mut Ufo, &Transform), Without<ShipProjectile>>,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (projectile_entity, projectile, mut projectile_transform) in projectiles.iter_mut() {
        for (ufo_entity, mut ufo, ufo_transform) in ufos.iter_mut() {
            let ufo_radius: f32 = 16.0;
            match *projectile {
                ShipProjectile::Rapid | ShipProjectile::Spread => {
                    if projectile_transform
                        .translation
                        .distance_squared(ufo_transform.translation)
                        < ufo_radius.powi(2)
                    {
                        commands.entity(projectile_entity).despawn();
                        if ufo.life > 0 {
                            ufo.life -= 1;
                        }
                    }
                }
                ShipProjectile::Plasma { mut power } => {
                    if projectile_transform
                        .translation
                        .distance_squared(ufo_transform.translation)
                        < (ufo_radius + power).powi(2)
                    {
                        let distance = projectile_transform
                            .translation
                            .distance(ufo_transform.translation);
                        let effect = (ufo_radius + power - distance).min(ufo.life as f32);
                        power -= effect;
                        if power <= 0.0 {
                            commands.entity(projectile_entity).despawn();
                        } else {
                            projectile_transform.scale = Vec3::splat(power / 16.0);
                        }
                        if ufo.life > 0 {
                            ufo.life -= effect.ceil() as i32;
                        }
                    }
                }
                ShipProjectile::Beam { .. } => {}
            }
            if ufo.life <= 0 {
                let velocity =
                    Vec2::from_angle(rand::random::<f32>() * std::f32::consts::TAU) * 30.0; // FIXME
                spawn_powerup(
                    rand::random(),
                    ufo_transform.translation.truncate(),
                    velocity,
                    5.0,
                    &mut commands,
                    &sprite_sheets.powerup,
                );
                commands.entity(ufo_entity).despawn();
                break;
            }
        }
    }
}

impl rand::distributions::Distribution<Powerup> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Powerup {
        use Powerup::*;
        match rng.gen_range(0..7) {
            0 => Laser,
            1 => Spread,
            2 => Beam,
            3 => Plasma,
            4 => ExtraLife,
            5 => LoseLife,
            6 => Shield,
            _ => unreachable!(),
        }
    }
}
fn spawn_powerup(
    powerup: Powerup,
    position: Vec2,
    velocity: Vec2,
    life: f32,
    commands: &mut Commands,
    sprite_sheet: &PowerupImages,
) {
    let texture = match powerup {
        Powerup::Laser => &sprite_sheet.laser,
        Powerup::Spread => &sprite_sheet.spread,
        Powerup::Beam => &sprite_sheet.beam,
        Powerup::Plasma => &sprite_sheet.plasma,
        Powerup::ExtraLife => &sprite_sheet.extra_life,
        Powerup::LoseLife => &sprite_sheet.lose_life,
        Powerup::Shield => &sprite_sheet.shield,
    }
    .clone();
    let transform = Transform::from_translation(position.extend(0.));
    commands
        .spawn(SpriteBundle {
            texture,
            transform,
            ..Default::default()
        })
        .insert(powerup)
        .insert(Moving {
            velocity,
            acceleration: Vec2::ZERO,
        })
        .insert(Expiring { life })
        .insert(Wrapping)
        .insert(LevelEntity);
}

fn ship_powerup_collision_system(
    mut commands: Commands,
    mut ships_query: Query<(&mut Ship, &Transform)>,
    powerups_query: Query<(Entity, &Powerup, &Transform)>,
) {
    for (mut ship, ship_transform) in ships_query.iter_mut() {
        for (powerup_entity, powerup, powerup_transform) in powerups_query.iter() {
            let distance_sq = ship_transform
                .translation
                .distance_squared(powerup_transform.translation);
            if distance_sq <= 32.0f32.powf(2.0) {
                match powerup {
                    Powerup::Laser => {
                        ship.weapon_rapid_level = (ship.weapon_rapid_level + 1).min(8)
                    }
                    Powerup::Spread => {
                        ship.weapon_spread_level = (ship.weapon_spread_level + 1).min(8)
                    }
                    Powerup::Beam => ship.weapon_beam_level = (ship.weapon_beam_level + 1).min(8),
                    Powerup::Plasma => {
                        ship.weapon_plasma_level = (ship.weapon_plasma_level + 1).min(8)
                    }
                    Powerup::ExtraLife => ship.lives += 1,
                    Powerup::LoseLife => ship.lives -= 1,
                    Powerup::Shield => ship.shield_level += 1,
                }
                commands.entity(powerup_entity).despawn();
            }
        }
    }
}

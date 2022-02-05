use bevy::{
    prelude::*,
    sprite,
    asset::LoadState
};

#[derive(Copy, Clone, Debug)]
enum AsteroidSize { Tiny = 0, Small, Medium, Large }

impl AsteroidSize {
    fn smaller(&self) -> Option<AsteroidSize> {
        match self {
            AsteroidSize::Tiny => None,
            AsteroidSize::Small => Some(AsteroidSize::Tiny),
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Large => Some(AsteroidSize::Large)
        }
    }
}

const ASTEROID_SIZES: usize = 4;
const ASTEROID_VARIANTS: usize = 12;

enum ShipWeapon {
    Rapid,
    Spread,
    Beam,
    Plasma
}

#[derive(Component)]
enum ShipProjectile {
    Rapid,
    Spread,
    Beam { power: u32 },
    Plasma { power: u32 }
}

#[derive(Component)]
struct Expiring {
    life: f32
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
    variant: usize
}

#[derive(Component, Default)]
struct Moving {
    velocity: Vec2,
    acceleration: Vec2
}

#[derive(Component)]
struct Spinning {
    speed: f32
}

#[derive(Component)]
struct Wrapping;

#[derive(Component, Default)]
struct Ship {
    throttle: bool,
    turn_left: bool,
    turn_right: bool,
    fire: bool,
    weapon: ShipWeapon,
    weapon_cooldown: f32
}

#[derive(Default)]
struct SpriteSheets {
    asteroids: Handle<TextureAtlas>,
    images: Vec<HandleUntyped>
}

struct GameState {
    level: u32
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    LoadLevel,
    InGame,
    Loading
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SpriteSheets::default())
        .insert_resource(GameState { level: 0 })
        .add_startup_system(init)
        .add_state(AppState::Loading)
        .add_system_set(SystemSet::on_update(AppState::Loading).with_system(loading))
        .add_system_set(SystemSet::on_enter(AppState::LoadLevel).with_system(load_level))
        .add_system_set(SystemSet::on_update(AppState::InGame)
            .with_system(ship_control_system)
            .with_system(ship_physics)
            .with_system(ship_sprite)
            .with_system(moving_system)
            .with_system(spinning_system)
            .with_system(wrapping_system)
            .with_system(expiring_system)
            .with_system(ship_projectile_asteroid_hit_system)
            .with_system(asteroid_split_system)
            )
        .run();
}

fn asteroid_texture_index(variant: usize, size: AsteroidSize) -> usize {
   variant * ASTEROID_SIZES  + size as usize
}

fn asteroid_sprite_rects() -> impl Iterator<Item=sprite::Rect> {
    let variant_rows = 5;
    let variant_sizes = [8, 16, 32, 48];
    let variant_width: u32 = variant_sizes.iter().sum();
    let variant_height = variant_sizes.into_iter().max().unwrap_or(0);

    (0..ASTEROID_VARIANTS).flat_map(move |variant_index| {
        let variant_x = (variant_index as u32/ variant_rows) * variant_width;
        let variant_y = (variant_index as u32 % variant_rows) * variant_height;

        variant_sizes.into_iter().scan(0, |size_x, size| {
            let result = (size_x.clone(), size);
            *size_x += size;
            Some(result)
        }).map(move |(size_x, size)| {

            sprite::Rect {
                min: Vec2::new((variant_x + size_x) as f32, variant_y as f32),
                max: Vec2::new((variant_x + size_x + size - 1) as f32, (variant_y + size - 1) as f32)
            }
        })
    })
}
fn init(mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut sprite_sheets: ResMut<SpriteSheets>) {

    sprite_sheets.images = asset_server.load_folder("img").unwrap();
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn loading(mut commands: Commands,
           asset_server: Res<AssetServer>,
           mut sprite_sheets: ResMut<SpriteSheets>,
           mut texture_atlases: ResMut<Assets<TextureAtlas>>,
           mut textures: ResMut<Assets<Image>>,
           mut state: ResMut<State<AppState>>,
           mut loading_text: Local<Option<Entity>>) {

    if loading_text.is_none() {
        *loading_text = Some(commands.spawn_bundle(TextBundle {
            text: Text::with_section("Loading...", TextStyle {
                font: asset_server.load("fonts/DejaVuSans.ttf"),
                font_size: 100.0,
                color: Color::WHITE
            }, TextAlignment::default()),
            style: Style {
                ..Default::default()
            },
            ..Default::default()
        }).id());
    }

    let handles = sprite_sheets.images.iter().map(|h| h.id);
    if let LoadState::Loaded = asset_server.get_group_load_state(handles)
    {
        // Initialize texture atlases
        let asteroid_texture = asset_server.load("img/asteroids.png");
        let mut asteroid_atlas = TextureAtlas::new_empty(asteroid_texture, Vec2::new(512.0, 256.0));

        for asteroid_rect in asteroid_sprite_rects() {
            asteroid_atlas.add_texture(asteroid_rect);
        }

        sprite_sheets.asteroids = texture_atlases.add(asteroid_atlas);

        // Loading finished
        if let Some(entity) = *loading_text {
            commands.entity(entity).despawn();
        }
        state.set(AppState::LoadLevel).unwrap();
    }
}

fn load_level(mut commands: Commands,
              asset_server: Res<AssetServer>,
              sprite_sheets: Res<SpriteSheets>,
              game_state: Res<GameState>,
              mut app_state: ResMut<State<AppState>>) {

    println!("setup level {}", game_state.level);

    let asteroid_variant = game_state.level as usize % ASTEROID_VARIANTS;
    let asteroid_data = (0..(game_state.level+3)).map(|i|
        (match i % 4 {
            0 => AsteroidSize::Large,
            1 => AsteroidSize::Medium,
            2 => AsteroidSize::Small,
            _ => AsteroidSize::Tiny
        }, i as f32 * 48.0));


    let background_texture = asset_server.load("img/background-1.png");
    commands
        .spawn_bundle(SpriteBundle {
            texture: background_texture,
            ..Default::default()
        });

    for (size, pos) in asteroid_data {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: sprite_sheets.asteroids.clone(),
                sprite: TextureAtlasSprite::new(asteroid_texture_index(asteroid_variant, size)),
                transform: Transform::from_xyz(24.0 + pos, pos * 2.0, 0.0),
                ..Default::default()
            })
        .insert(Moving { velocity: Vec2::new(13.5, 15.7), ..Default::default() })
            .insert(Spinning { speed: 0.2 })
            .insert(Wrapping)
            .insert(Asteroid { size, integrity: size as i32 * 4 + 1, variant: asteroid_variant })
            ;
    }

    let ship = Ship::default();
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(ship_texture_path(&ship)),
            ..Default::default()
        })
    .insert(Moving::default())
        .insert(Wrapping)
        .insert(ship)
        ;

    app_state.set(AppState::InGame).unwrap();
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

fn expiring_system(mut commands: Commands, mut expiring_query: Query<(Entity, &mut Expiring)>, time: Res<Time>) {
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
    
    for mut ship in ship_query.iter_mut() {
        ship.throttle = throttle;
        ship.turn_left = turn_left;
        ship.turn_right = turn_right;
        ship.fire = fire;
    }
}

fn ship_physics(mut commands: Commands, 
                asset_server: Res<AssetServer>,
                mut ship_query: Query<(&mut Ship, &mut Moving, &mut Transform)>,
                time: Res<Time>) {
    let time_delta = time.delta().as_secs_f32();

    for (mut ship, mut moving, mut transform) in ship_query.iter_mut() {
        let angular_velocity = match (ship.turn_left, ship.turn_right) {
            (true, false) => 3.0,
            (false, true) => -3.0,
            _ => 0.0
        };
        let acceleration = if ship.throttle { 50.0 } else { 0.0 };
        transform.rotation *= Quat::from_rotation_z(angular_velocity * time_delta); 
        moving.acceleration = (transform.rotation * Vec3::Y * acceleration).truncate();

        if ship.weapon_cooldown > 0.0 {
            ship.weapon_cooldown -= time_delta;
        }

        if ship.fire && ship.weapon_cooldown <= 0.0 {
            ship.weapon_cooldown = 0.3;

            let projectile = match ship.weapon {
                ShipWeapon::Rapid => ShipProjectile::Rapid,
                ShipWeapon::Spread => ShipProjectile::Spread,
                ShipWeapon::Beam => ShipProjectile::Beam { power: 20 },
                ShipWeapon::Plasma => ShipProjectile::Plasma { power: 100 },
            };
            let texture_path = match ship.weapon {
                ShipWeapon::Rapid => "img/laser.png",
                ShipWeapon::Spread => "img/shot.png",
                ShipWeapon::Beam => "img/continuous_beam.png",
                ShipWeapon::Plasma => "img/plasma.png",
            };
            let texture = asset_server.load(texture_path);
            let velocity = (transform.rotation * Vec3::Y * 500.0).truncate();

            commands
                .spawn_bundle(SpriteBundle {
                    texture: texture,
                    transform: Transform {
                        translation: transform.translation.clone(),
                        rotation: transform.rotation.clone(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Moving { velocity, ..Default::default() })
                .insert(Wrapping)
                .insert(projectile)
                .insert(Expiring { life: 1.0 })
                ;

        }
    }
}

fn ship_texture_path(ship: &Ship) -> &'static str {
    match(&ship.weapon, ship.throttle, ship.turn_left, ship.turn_right) {
        (ShipWeapon::Rapid, true, false, false) => "img/ship-rapid_accelerating.png",
        (ShipWeapon::Rapid, false, true, false) => "img/ship-rapid_left.png",
        (ShipWeapon::Rapid, true, true, false) => "img/ship-rapid_left_accelerating.png",
        (ShipWeapon::Rapid, false, false, true) => "img/ship-rapid_right.png",
        (ShipWeapon::Rapid, true, false, true) => "img/ship-rapid_right_accelerating.png",
        (ShipWeapon::Rapid, _, _, _) => "img/ship-rapid.png",
        (ShipWeapon::Spread, true, false, false) => "img/ship-spread_accelerating.png",
        (ShipWeapon::Spread, false, true, false) => "img/ship-spread_left.png",
        (ShipWeapon::Spread, true, true, false) => "img/ship-spread_left_accelerating.png",
        (ShipWeapon::Spread, false, false, true) => "img/ship-spread_right.png",
        (ShipWeapon::Spread, true, false, true) => "img/ship-spread_right_accelerating.png",
        (ShipWeapon::Spread, _, _, _) => "img/ship-spread.png",
        (ShipWeapon::Beam, true, false, false) => "img/ship-beam_accelerating.png",
        (ShipWeapon::Beam, false, true, false) => "img/ship-beam_left.png",
        (ShipWeapon::Beam, true, true, false) => "img/ship-beam_left_accelerating.png",
        (ShipWeapon::Beam, false, false, true) => "img/ship-beam_right.png",
        (ShipWeapon::Beam, true, false, true) => "img/ship-beam_right_accelerating.png",
        (ShipWeapon::Beam, _, _, _) => "img/ship-beam.png",
        (ShipWeapon::Plasma, true, false, false) => "img/ship-plasma_accelerating.png",
        (ShipWeapon::Plasma, false, true, false) => "img/ship-plasma_left.png",
        (ShipWeapon::Plasma, true, true, false) => "img/ship-plasma_left_accelerating.png",
        (ShipWeapon::Plasma, false, false, true) => "img/ship-plasma_right.png",
        (ShipWeapon::Plasma, true, false, true) => "img/ship-plasma_right_accelerating.png",
        (ShipWeapon::Plasma, _, _, _) => "img/ship-plasma.png"
    }
}
fn ship_sprite(mut ship_query: Query<(&Ship, &mut Handle<Image>)>,
               asset_server: Res<AssetServer>) {
    for (ship, mut image) in ship_query.iter_mut() {
        *image = asset_server.load(ship_texture_path(&ship));
    }
}

fn ship_projectile_asteroid_hit_system(mut commands: Commands,
                                       projectiles: Query<(Entity, &ShipProjectile, &Transform)>,
                                       mut asteroids: Query<(&mut Asteroid, &Transform)>) {
    for (projectile_entity, _projectile, projectile_transform) in projectiles.iter() {
        for (mut asteroid, asteroid_transform) in asteroids.iter_mut() {
            let asteroid_radius: f32 = match asteroid.size {
                AsteroidSize::Tiny => 4.0,
                AsteroidSize::Small => 8.0,
                AsteroidSize::Medium => 16.0,
                AsteroidSize::Large => 24.0
            };
            if projectile_transform.translation.distance_squared(asteroid_transform.translation) < asteroid_radius.powi(2)  {
                commands.entity(projectile_entity).despawn();
                if asteroid.integrity > 0 {
                    asteroid.integrity -= 1;
                }
                
            }
        }
    }
}

fn asteroid_split_system(mut commands: Commands,
                         asteroids: Query<(Entity, &Asteroid, &Transform)>,
                         sprite_sheets: Res<SpriteSheets>) {
    for (asteroid_entity, asteroid, transform) in asteroids.iter() {
        if asteroid.integrity == 0 {
            commands.entity(asteroid_entity).despawn();
            if let Some(size) = asteroid.size.smaller() {
                let direction = (transform.rotation * transform.translation).truncate().normalize();
                let data = [direction, -direction];
                let transform = transform.clone();

                for dir in data {
                    commands
                        .spawn_bundle(SpriteSheetBundle {
                            texture_atlas: sprite_sheets.asteroids.clone(),
                            sprite: TextureAtlasSprite { index: asteroid_texture_index(asteroid.variant, size), ..Default::default() },
                            transform,
                            ..Default::default()
                        })
                    .insert(Moving { velocity: dir * 30.0, ..Default::default() })
                        .insert(Spinning { speed: 0.2 })
                        .insert(Wrapping)
                        .insert(Asteroid { size, integrity: size as i32 * 4 + 1, variant: asteroid.variant  });

                    }
            }
        }
    }
}

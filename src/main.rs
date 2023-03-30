use bevy::{asset::LoadState, prelude::*};
use rand::{random, thread_rng, Rng};

mod bundles;
mod components;
mod constants;
mod resources;
mod utils;

use crate::{bundles::*, components::*, constants::*, resources::*, utils::*};

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
            level: Level(0),
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
                shield_sprite,
                moving_system,
                spinning_system,
                wrapping_system,
                expiring_system,
                animation_system,
                asteroid_split_system,
                ufo_spawn_system,
                ufo_movement_system,
                ufo_animation_system,
                ufo_shoot_system,
            )
                .in_set(OnUpdate(AppState::InGame)),
        )
        .add_systems(
            (
                ship_projectile_asteroid_hit_system,
                ship_projectile_ufo_hit_system,
                ship_powerup_collision_system,
                ship_asteroid_collision_system,
                ship_ufo_laser_collision_system,
                level_finished_system,
            )
                .in_set(OnUpdate(AppState::InGame)),
        )
        .add_systems(
            (
                update_hud_system,
                update_hud_text_system.after(update_hud_system),
            )
                .in_set(OnUpdate(AppState::InGame)),
        )
        .add_system(despawn_tagged::<LevelEntity>.in_schedule(OnExit(AppState::InGame)))
        .run();
}

fn despawn_tagged<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
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
        let font = asset_server.load("fonts/DejaVuSans.ttf");
        *loading_text = Some(commands.spawn(LoadingTextBundle::new(font)).id());
    }

    let handles = sprite_sheets.images.iter().map(|h| h.id());
    if let LoadState::Loaded = asset_server.get_group_load_state(handles) {
        // Initialize texture atlases
        let asteroid_texture = asset_server.load("img/asteroids.png");
        let mut asteroid_atlas = TextureAtlas::new_empty(asteroid_texture, Vec2::new(512.0, 256.0));

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
            shield: asset_server.load("img/shield.png"),
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

        sprite_sheets.explosion.normal = (1..=EXPLOSION_IMAGES)
            .map(|i| format!("img/explosion/explosion1_{i:04}.png"))
            .map(|path| asset_server.load(&path))
            .collect();

        // Loading finished
        if let Some(entity) = *loading_text {
            commands.entity(entity).despawn();
        }
        next_state.set(AppState::LoadLevel);
    }
}

fn load_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sprite_sheets: Res<SpriteSheets>,
    game_state: Res<GameState>,
    mut app_state: ResMut<NextState<AppState>>,
    mut ships_query: Query<&mut Transform, With<Ship>>,
) {
    println!("setup level {}", game_state.level.0);

    let asteroid_variant = game_state.level.asteroid_variant();

    let background_texture = asset_server.load(&format!(
        "img/background-{}.png",
        game_state.level.background_image()
    ));
    commands
        .spawn(SpriteBundle {
            texture: background_texture,
            transform: Transform::from_xyz(0.0, 0.0, -0.01),
            ..Default::default()
        })
        .insert(LevelEntity);

    let mut rng = thread_rng();
    for size in game_state.level.asteroids() {
        let distance: f32 = rng.gen_range(game_state.level.asteroid_distance_bounds());
        let direction = random::<f32>() * std::f32::consts::TAU;
        let position: Vec2 = Vec2::from_angle(direction) * distance;
        let heading = random::<f32>() * std::f32::consts::TAU;
        let speed = rng.gen_range(game_state.level.asteroid_speed_bounds());
        let velocity = Vec2::from_angle(heading) * speed;
        let spinning_speed = 0.2;
        commands.spawn(AsteroidBundle::new(
            sprite_sheets.as_ref(),
            asteroid_variant,
            size,
            position,
            velocity,
            spinning_speed,
        ));
    }

    if ships_query.is_empty() {
        commands
            .spawn(ShipBundle::new(sprite_sheets.as_ref()))
            .with_children(|ship| {
                ship.spawn(ShipShieldBundle::new(&sprite_sheets.ship));
            });
    } else {
        for mut transform in ships_query.iter_mut() {
            transform.translation = Vec3::ZERO;
        }
    }

    commands
        .spawn(TextBundle::from_section(
            "",
            TextStyle {
                font: asset_server.load("fonts/DejaVuSans.ttf"),
                font_size: 20.0,
                color: Color::WHITE,
            },
        ))
        .insert(HUD::default())
        .insert(LevelEntity);
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

fn ship_physics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ship_query: Query<(Entity, &mut Ship, &mut Moving, &mut Transform)>,
    mut beam_query: Query<(&mut Beam, &mut Transform), Without<Ship>>,
    time: Res<Time>,
) {
    let time_delta = time.delta().as_secs_f32();

    for (ship_entity, mut ship, mut moving, mut transform) in ship_query.iter_mut() {
        ship.invulnerability = (ship.invulnerability - time_delta).max(0.);
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
                    commands.spawn(ShipProjectileBundle::new(
                        projectile,
                        texture.clone(),
                        velocity.clone(),
                        left_transform,
                        0.25,
                    ));
                    commands.spawn(ShipProjectileBundle::new(
                        projectile,
                        texture,
                        velocity,
                        right_transform,
                        0.25,
                    ));
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
                        commands.spawn(ShipProjectileBundle::new(
                            projectile,
                            texture.clone(),
                            velocity,
                            transform,
                            0.20,
                        ));
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
                    commands.spawn(ShipProjectileBundle::new(
                        projectile, texture, velocity, transform, 0.5,
                    ));
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

fn ship_sprite(
    mut ship_query: Query<(&Ship, &mut Sprite, &mut Handle<Image>)>,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (ship, mut sprite, mut image) in ship_query.iter_mut() {
        *image = sprite_sheets.ship.choose(&ship);
        let alpha = if ship.invulnerability > 0.0 { 0.5 } else { 1.0 };
        sprite.color.set_a(alpha);
    }
}

fn shield_sprite(
    mut shield_query: Query<(&Parent, &mut Visibility), With<ShipShield>>,
    ship_query: Query<&Ship>,
) {
    for (parent, mut visibility) in shield_query.iter_mut() {
        let ship = ship_query.get(parent.get());
        if ship
            .expect("ShipShield should have a Ship parent")
            .shield_level
            > 0
        {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
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
                let data = [direction, -direction]
                    .into_iter()
                    .cycle()
                    .take(game_state.level.asteroid_frag_count() as usize);

                let position = transform.translation.truncate();
                let spinning_speed = 0.2;
                for dir in data {
                    let velocity = dir * 30.0;
                    commands.spawn(AsteroidBundle::new(
                        sprite_sheets.as_ref(),
                        asteroid.variant,
                        size,
                        position,
                        velocity,
                        spinning_speed,
                    ));
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
        game_state.level.increment();
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
        let horizontal: bool = random();
        let direction: bool = random();
        let span = if horizontal { 800.0 } else { 480.0 };
        let d = random::<f32>() * span;
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

        let ufo = Ufo {
            start_position: position,
            end_position: -position,
            frequency: random::<f32>() * 5.0,
            amplitude: random::<f32>() * 90.0 + 10.0,
            duration: game_state.level.ufo_duration(),
            time: 0.0,
            shoot_delay: game_state.level.ufo_shoot_delay(),
            shoot_accuracy: game_state.level.ufo_shoot_accuracy(),
            life: 20,
        };
        commands.spawn(UfoBundle::new(&sprite_sheets.ufo, ufo));
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
                (1.0 - ufo.shoot_accuracy) * (random::<f32>() - 0.5) * std::f32::consts::PI;
            let aim = Vec2::from_angle(aim_error).rotate(target);
            let speed = 500.0; // FIXME
            let velocity = aim * speed;
            let angle = Vec2::Y.angle_between(aim);
            let life = 2.0;
            commands.spawn(UfoLaserBundle::new(
                &sprite_sheets.ufo,
                ufo_transform.translation.truncate(),
                angle,
                velocity,
                life,
            ));
        }
    }
}
fn random_ufo_interval() -> u32 {
    const MIN: f32 = 400.0;
    const MAX: f32 = 800.0;
    (random::<f32>() * (MAX - MIN) + MIN) as u32
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
                let velocity = Vec2::from_angle(random::<f32>() * std::f32::consts::TAU) * 30.0; // FIXME
                commands.spawn(PowerupBundle::new(
                    random(),
                    ufo_transform.translation.truncate(),
                    velocity,
                    5.0,
                    &sprite_sheets.powerup,
                ));
                commands.spawn(ExplosionBundle::new(
                    &sprite_sheets.explosion,
                    ufo_transform.translation.truncate(),
                ));
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
                    Powerup::LoseLife => ship.lives = ship.lives.max(1) - 1,
                    Powerup::Shield => ship.shield_level += 1,
                }
                commands.entity(powerup_entity).despawn();
            }
        }
    }
}

fn ship_asteroid_collision_system(
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
    mut ships_query: Query<(&mut Ship, &mut Transform, &mut Moving, &CollisionShape)>,
    asteroids_query: Query<(&Transform, &Moving, &CollisionShape), (With<Asteroid>, Without<Ship>)>,
) {
    for (mut ship, mut ship_transform, mut ship_moving, ship_shape) in ships_query.iter_mut() {
        if ship.invulnerability > 0.0 {
            continue;
        }
        let ship_position = ship_transform.translation.truncate();
        for (asteroid_transform, asteroid_moving, asteroid_shape) in asteroids_query.iter() {
            let asteroid_position = asteroid_transform.translation.truncate();
            if ship_shape.intersects(ship_position, asteroid_shape, asteroid_position) {
                if ship.shield_level > 0 {
                    ship.shield_level -= 1;
                    let diff = ship_position - asteroid_position;
                    let speed = (asteroid_moving.velocity.project_onto(diff)
                        - ship_moving.velocity)
                        .length();
                    ship_moving.velocity = diff.normalize() * speed;
                } else {
                    ship_transform.translation = Vec3::ZERO;
                    ship.lives = ship.lives.max(1) - 1; //FIXME
                    ship.invulnerability = SHIP_INVULNERABILITY;
                    commands.spawn(ExplosionBundle::new(
                        &sprite_sheets.explosion,
                        ship_position,
                    ));
                }
            }
        }
    }
}

fn ship_ufo_laser_collision_system(
    mut commands: Commands,
    mut ships_query: Query<(&mut Ship, &mut Transform, &mut Moving)>,
    ufo_laser_query: Query<(&Transform, &Moving), (With<UfoLaser>, Without<Ship>)>,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (mut ship, mut ship_transform, mut ship_moving) in ships_query.iter_mut() {
        if ship.invulnerability > 0.0 {
            continue;
        }
        let ship_position = ship_transform.translation.truncate();
        for (laser_transform, laser_moving) in ufo_laser_query.iter() {
            let laser_position = laser_transform.translation.truncate();
            let laser_radius: f32 = 1.0;
            let ship_radius: f32 = 16.0;
            let distance_sq = ship_position.distance_squared(laser_position);
            if distance_sq <= (laser_radius + ship_radius).powf(2.0) {
                if ship.shield_level > 0 {
                    ship.shield_level -= 1;
                    let diff = ship_position - laser_position;
                    let speed =
                        (laser_moving.velocity.project_onto(diff) - ship_moving.velocity).length();
                    ship_moving.velocity = diff.normalize() * speed;
                } else {
                    ship_transform.translation = Vec3::ZERO;
                    ship.lives = ship.lives.max(1) - 1; //FIXME
                    ship.invulnerability = SHIP_INVULNERABILITY;
                    commands.spawn(ExplosionBundle::new(
                        &sprite_sheets.explosion,
                        ship_position,
                    ));
                }
            }
        }
    }
}

fn update_hud_system(
    ships_query: Query<&Ship>,
    game_state: Res<GameState>,
    mut hud_query: Query<&mut HUD>,
) {
    let ship = ships_query.single();
    for mut hud in hud_query.iter_mut() {
        let new_hud = HUD {
            level: game_state.level.0,
            score: game_state.score,
            lives: ship.lives,
            weapon: ship.weapon,
            weapon_rapid_level: ship.weapon_rapid_level,
            weapon_spread_level: ship.weapon_spread_level,
            weapon_beam_level: ship.weapon_beam_level,
            weapon_plasma_level: ship.weapon_plasma_level,
        };
        if *hud != new_hud {
            *hud = new_hud;
        }
    }
}

fn update_hud_text_system(mut hud_query: Query<(&HUD, &mut Text), Changed<HUD>>) {
    for (hud, mut text) in hud_query.iter_mut() {
        fn weapon_text(name: &str, level: u8, selected: bool) -> String {
            match (level, selected) {
                (0, _) => String::new(),
                (level, true) => format!("[{name}{level}]"),
                (level, false) => format!("{name}{level}"),
            }
        }
        let weapons = [
            (ShipWeapon::Rapid, "L", hud.weapon_rapid_level),
            (ShipWeapon::Spread, "S", hud.weapon_spread_level),
            (ShipWeapon::Beam, "B", hud.weapon_beam_level),
            (ShipWeapon::Plasma, "P", hud.weapon_plasma_level),
        ]
        .map(|(weapon, name, level)| weapon_text(name, level, weapon == hud.weapon));

        let hud_text = format!(
            "Level: {} | Score: {} | Lives: {} | Weapons: {}",
            hud.level,
            hud.score,
            hud.lives,
            &weapons.join(" ")
        );
        dbg!(&hud_text);
        text.sections[0].value = hud_text;
    }
}

fn animation_system(
    mut animated_query: Query<(&mut Animated, &mut Handle<Image>)>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    for (mut animated, mut image) in animated_query.iter_mut() {
        animated.elapsed += delta;
        let position = if animated.looping {
            animated.elapsed.rem_euclid(animated.animation.duration)
        } else {
            animated.elapsed.min(animated.animation.duration)
        };
        let frame = ((animated.animation.frames.len() - 1) as f32 * position
            / animated.animation.duration)
            .floor() as usize;

        *image = animated.animation.frames[frame].clone()
    }
}

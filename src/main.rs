use bevy::{asset::LoadState, prelude::*};
use rand::{random, thread_rng, Rng};

mod bundles;
mod components;
mod constants;
mod plugins;
mod resources;
mod utils;

use crate::{bundles::*, components::*, constants::*, resources::*, utils::*};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Title,
    NewGame,
    LoadLevel,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SpriteSheets::default())
        .insert_resource(Level(0))
        .insert_resource(Score(0))
        .insert_resource(LevelStartDelayTimer::default())
        .add_system(init.on_startup())
        .add_state::<AppState>()
        .add_plugin(plugins::CameraPlugin)
        .add_plugin(plugins::TitleScreenPlugin)
        .add_system(loading.in_set(OnUpdate(AppState::Loading)))
        .add_system(new_game.in_schedule(OnEnter(AppState::NewGame)))
        .add_system(load_level.in_schedule(OnEnter(AppState::LoadLevel)))
        .add_systems(
            (
                level_start_delay_system,
                scaling_system,
                expiring_system,
                fading_system,
            )
                .in_set(OnUpdate(AppState::LoadLevel)),
        )
        .add_systems(
            (
                ship_control_system,
                ship_physics,
                ship_sprite,
                ship_respawn_system,
                shield_sprite,
                moving_system,
                spinning_system,
                wrapping_system,
                expiring_system,
                scaling_system,
                fading_system,
                animation_system,
                collision_shape_system,
                beam_sprite_system,
            )
                .in_set(OnUpdate(AppState::InGame)),
        )
        .add_systems(
            (
                asteroid_hit_system,
                asteroid_split_system,
                ship_projectile_asteroid_hit_system.after(ship_physics),
                ship_powerup_collision_system,
                ship_asteroid_collision_system,
                level_finished_system,
                gameover_system,
                cheat_system,
            )
                .in_set(OnUpdate(AppState::InGame)),
        )
        .add_system(despawn_tagged::<LevelEntity>.in_schedule(OnExit(AppState::InGame)))
        .add_plugin(plugins::HudPlugin)
        .add_plugin(plugins::UfoPlugin)
        .run();
}

fn despawn_tagged<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn init(asset_server: Res<AssetServer>, mut sprite_sheets: ResMut<SpriteSheets>) {
    sprite_sheets.images = asset_server.load_folder("img").unwrap();
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

        sprite_sheets.particles = ParticleImages {
            spark: asset_server.load("img/spark.png"),
            corona: asset_server.load("img/flares/corona.png"),
            ring: asset_server.load("img/flares/tunelring-alpha.png"),
            wave: asset_server.load("img/flares/wave.png"),
        };
        // Loading finished
        if let Some(entity) = *loading_text {
            commands.entity(entity).despawn();
        }
        next_state.set(AppState::Title);
    }
}

fn new_game(
    mut level: ResMut<Level>,
    mut score: ResMut<Score>,
    ships_query: Query<Entity, With<Ship>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    *level = Level(0);
    *score = Score(0);
    for ship_entity in ships_query.iter() {
        commands.entity(ship_entity).despawn_recursive();
    }
    next_state.set(AppState::LoadLevel);
}

fn load_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sprite_sheets: Res<SpriteSheets>,
    level: Res<Level>,
    mut ships_query: Query<(&mut Transform, &mut Moving), With<Ship>>,
    mut level_start_delay_timer: ResMut<LevelStartDelayTimer>,
) {
    println!("setup level {}", level.number());

    let asteroid_variant = level.asteroid_variant();

    let background_texture =
        asset_server.load(&format!("img/background-{}.png", level.background_image()));
    commands
        .spawn(SpriteBundle {
            texture: background_texture,
            transform: Transform::from_xyz(0.0, 0.0, -0.09),
            ..Default::default()
        })
        .insert(LevelEntity);

    let mut rng = thread_rng();
    for size in level.asteroids() {
        let distance: f32 = rng.gen_range(level.asteroid_distance_bounds());
        let direction = random::<f32>() * std::f32::consts::TAU;
        let position: Vec2 = Vec2::from_angle(direction) * distance;
        let heading = random::<f32>() * std::f32::consts::TAU;
        let speed = rng.gen_range(level.asteroid_speed_bounds());
        let velocity = Vec2::from_angle(heading) * speed;
        let spinning_speed = random::<f32>() - 0.5;
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
        let ship = Ship {
            weapon_rapid_level: 1,
            shield_level: 0,
            lives: 3,
            ..Ship::default()
        };
        commands
            .spawn(ShipBundle::new(ship, sprite_sheets.as_ref()))
            .with_children(|ship| {
                ship.spawn(ShipShieldBundle::new(&sprite_sheets.ship));
                let projectile = ShipProjectile::Beam { power: 20.0 };
                let beam_from = Vec2::ZERO;
                let length = 0.0;
                let max_length = 0.0;
                let texture = asset_server.load("img/continuous_beam.png");
                let mut transform = Transform::from_xyz(0.0, 0.0, -0.01);
                transform.scale.y = length / 128.0;
                ship.spawn(ShipBeamBundle::new(
                    projectile, texture, transform, beam_from, length, max_length,
                ))
                .with_children(|beam| {
                    beam.spawn(SpriteBundle {
                        texture: asset_server.load("img/continuous_tip.png"),
                        transform: Transform::from_xyz(0.0, 128.0, 0.0),
                        ..Default::default()
                    })
                    .insert(BeamTip);
                });
            });
    } else {
        for (mut transform, mut moving) in ships_query.iter_mut() {
            transform.translation = Vec3::ZERO;
            moving.velocity = Vec2::ZERO;
            moving.acceleration = Vec2::ZERO;
        }
    }

    commands.spawn(GameNotificationBundle::new(
        format!("Level {}", level.number()),
        asset_server.load("fonts/DejaVuSans.ttf"),
        Vec2::ZERO,
        60.0,
        3.0,
    ));

    *level_start_delay_timer =
        LevelStartDelayTimer(Timer::from_seconds(LEVEL_START_DELAY, TimerMode::Once));
}
fn level_start_delay_system(
    mut timer: ResMut<LevelStartDelayTimer>,
    time: Res<Time>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        app_state.set(AppState::InGame);
    }
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
fn scaling_system(mut scaling_query: Query<(&mut Scaling, &mut Transform)>, time: Res<Time>) {
    for (mut scaling, mut transform) in scaling_query.iter_mut() {
        scaling.elapsed += time.delta_seconds();
        let scale = lerp(scaling.from, scaling.to, scaling.elapsed / scaling.duration);
        transform.scale = Vec3::splat(scale);
    }
}
fn fading_system(
    mut fading_query: Query<(&mut Fading, Option<&mut Text>, Option<&mut Sprite>)>,
    time: Res<Time>,
) {
    for (mut fading, text, sprite) in fading_query.iter_mut() {
        fading.elapsed += time.delta_seconds();
        let alpha = lerp(fading.from, fading.to, fading.elapsed / fading.duration);
        if let Some(mut text) = text {
            for section in text.sections.iter_mut() {
                section.style.color.set_a(alpha);
            }
        }
        if let Some(mut sprite) = sprite {
            sprite.color.set_a(alpha);
        }
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
fn ship_respawn_system(
    mut ships_query: Query<(&mut Ship, &mut Transform, &mut Moving, &mut Visibility)>,
    time: Res<Time>,
) {
    for (mut ship, mut transform, mut moving, mut visibility) in ships_query.iter_mut() {
        if ship.lives > 0 && ship.respawn_delay > 0.0 {
            ship.respawn_delay -= time.delta_seconds();
            if ship.respawn_delay > 0.0 {
                *visibility = Visibility::Hidden;
                ship.invulnerability = 100.0;
            } else {
                *visibility = Visibility::Visible;
                ship.invulnerability = SHIP_INVULNERABILITY;
                transform.translation = Vec3::ZERO;
                moving.velocity = Vec2::ZERO;
            }
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
        if ship.respawn_delay > 0.0 {
            ship.fire = false;
            continue;
        }
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
    mut ship_query: Query<(&mut Ship, &mut Moving, &mut Transform)>,
    mut beam_query: Query<&mut Beam, Without<Ship>>,
    time: Res<Time>,
) {
    let time_delta = time.delta().as_secs_f32();

    for (mut ship, mut moving, mut transform) in ship_query.iter_mut() {
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
            match ship.weapon {
                ShipWeapon::Rapid => {
                    let projectile = ShipProjectile::Rapid;
                    let texture = asset_server.load("img/laser.png");
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
                        1.0,
                    ));
                    commands.spawn(ShipProjectileBundle::new(
                        projectile,
                        texture,
                        velocity,
                        right_transform,
                        0.25,
                        1.0,
                    ));
                    ship.weapon_cooldown =
                        lerp(0.3, 0.05, (ship.weapon_rapid_level - 1) as f32 / 8.0);
                }
                ShipWeapon::Spread => {
                    let projectile = ShipProjectile::Spread;
                    let texture = asset_server.load("img/shot.png");
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
                            1.0,
                        ));
                    }
                    ship.weapon_cooldown =
                        lerp(0.8, 0.3, (ship.weapon_spread_level - 1) as f32 / 8.0);
                }
                ShipWeapon::Plasma => {
                    let projectile = ShipProjectile::Plasma {
                        power: lerp(4.0, 20.0, (ship.weapon_plasma_level - 1) as f32 / 8.0),
                    };
                    let texture = asset_server.load("img/plasma.png");
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
                        projectile, texture, velocity, transform, 0.5, power,
                    ));
                    ship.weapon_cooldown =
                        lerp(1.2, 0.8, (ship.weapon_plasma_level - 1) as f32 / 8.0);
                }
                ShipWeapon::Beam => {
                    for mut beam in beam_query.iter_mut() {
                        beam.active = true;
                        beam.sustained += time_delta;
                        if beam.sustained > BEAM_EXTEND_TIME {
                            beam.max_length =
                                (beam.max_length - time_delta * BEAM_SHRINK_RATE).max(0.);
                        }
                        if beam.cooldown <= 0.0 {
                            beam.length = beam
                                .max_length
                                .min(beam.length + beam.max_length * time_delta / BEAM_EXTEND_TIME);
                        } else {
                            beam.cooldown -= time_delta;
                        }
                    }
                }
            }
        } else if matches!(ship.weapon, ShipWeapon::Beam) {
            for mut beam in beam_query.iter_mut() {
                beam.active = false;
                if beam.length > 0.0 {
                    beam.length = (beam.length - time_delta * BEAM_RETRACT_RATE).max(0.0);
                } else {
                    beam.sustained = 0.0;
                    let max_length =
                        BEAM_BASE_LENGTH + BEAM_LENGTH_PER_LEVEL * ship.weapon_beam_level as f32;
                    beam.max_length =
                        (beam.max_length + time_delta * BEAM_RECHARGE_RATE).min(max_length);
                }
            }
        }
    }
}

fn beam_sprite_system(
    mut beam_query: Query<(&Beam, &mut Transform, &Children), Without<BeamTip>>,
    mut tip_query: Query<&mut Transform, With<BeamTip>>,
) {
    for (beam, mut transform, children) in beam_query.iter_mut() {
        transform.scale.y = beam.length / 128.0;
        for child in children.iter() {
            if let Ok(mut tip_transform) = tip_query.get_mut(*child) {
                tip_transform.scale.y = 1.0 / transform.scale.y;
            }
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
    mut projectiles: Query<(
        Entity,
        &mut ShipProjectile,
        &mut Transform,
        &mut CollisionShape,
        Option<&mut Beam>,
    )>,
    mut asteroids: Query<(&mut Asteroid, &CollisionShape, &Transform), Without<ShipProjectile>>,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (
        projectile_entity,
        projectile,
        mut projectile_transform,
        mut projectile_shape,
        mut maybe_beam,
    ) in projectiles.iter_mut()
    {
        for (mut asteroid, asteroid_shape, asteroid_transform) in asteroids.iter_mut() {
            if projectile_shape.intersects(asteroid_shape) {
                match *projectile {
                    ShipProjectile::Rapid | ShipProjectile::Spread => {
                        commands.entity(projectile_entity).despawn();
                        if asteroid.integrity > 0 {
                            asteroid.integrity -= 1;
                        }
                    }
                    ShipProjectile::Plasma { mut power } => {
                        let overlap = -projectile_shape.distance(asteroid_shape).min(0.0);
                        let effect = overlap.min(asteroid.integrity as f32);
                        power -= effect;
                        *projectile_shape = CollisionShape::new(
                            Shape::Circle {
                                center: Vec2::ZERO,
                                radius: power,
                            },
                            *projectile_transform,
                        );
                        if power <= 0.0 {
                            commands.entity(projectile_entity).despawn();
                        } else {
                            projectile_transform.scale = Vec3::splat(power / 16.0);
                        }
                        if asteroid.integrity > 0 {
                            asteroid.integrity -= effect.ceil() as i32;
                        }
                    }
                    ShipProjectile::Beam { .. } => {
                        if let Some(ref mut beam) = maybe_beam {
                            if beam.active {
                                beam.length = projectile_shape
                                    .distance(asteroid_shape)
                                    .min(beam.max_length);
                                if beam.cooldown <= 0.0 {
                                    asteroid.integrity -= BEAM_DAMAGE_PER_HIT;
                                    beam.cooldown = BEAM_HIT_INTERVAL;
                                }
                            }
                        }
                    }
                }
                let point = projectile_shape.collision_point(asteroid_shape);
                let direction = (point - asteroid_transform.translation.truncate()).normalize();
                for _ in 0..10 {
                    let speed = lerp(10.0, 100.0, random());
                    let velocity =
                        (direction + (direction.perp() * lerp(-0.5, 0.5, random()))) * speed;
                    let acceleration = Vec2::ZERO;
                    commands.spawn(SparkParticleBundle::new(
                        point,
                        velocity,
                        acceleration,
                        &sprite_sheets.particles,
                    ));
                }
            }
        }
    }
}

fn asteroid_hit_system(
    mut asteroids_query: Query<(&mut Moving, &CollisionShape, &Transform), With<Asteroid>>,
) {
    let mut pairs = asteroids_query.iter_combinations_mut();
    while let Some([(mut a_moving, a_shape, a_transform), (mut b_moving, b_shape, b_transform)]) =
        pairs.fetch_next()
    {
        if a_shape.intersects(b_shape) {
            let direction = (a_transform.translation - b_transform.translation)
                .truncate()
                .normalize();
            a_moving.velocity = direction * a_moving.velocity.length();
            b_moving.velocity = -direction * b_moving.velocity.length();
        }
    }
}
fn asteroid_split_system(
    mut commands: Commands,
    asteroids: Query<(Entity, &Asteroid, &Transform)>,
    sprite_sheets: Res<SpriteSheets>,
    mut score: ResMut<Score>,
    level: Res<Level>,
    asset_server: Res<AssetServer>,
) {
    for (asteroid_entity, asteroid, transform) in asteroids.iter() {
        if asteroid.integrity <= 0 {
            let score_delta = asteroid_score(asteroid.size);
            score.increase(score_delta);
            commands.spawn(GameNotificationBundle::new(
                format!("{}", score_delta),
                asset_server.load("fonts/DejaVuSans.ttf"),
                transform.translation.truncate(),
                20.0,
                1.0,
            ));
            commands.spawn(CoronaParticleBundle::new(
                transform.translation.truncate(),
                asteroid.size.radius() / AsteroidSize::Large.radius(),
                &sprite_sheets.particles,
            ));
            commands.entity(asteroid_entity).despawn();
            if let Some(size) = asteroid.size.smaller() {
                let direction = (transform.rotation * transform.translation)
                    .truncate()
                    .normalize();
                let data = [direction, -direction]
                    .into_iter()
                    .cycle()
                    .take(level.asteroid_frag_count() as usize);

                let position = transform.translation.truncate();
                let spinning_speed = random::<f32>() - 0.5;
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
    mut level: ResMut<Level>,
    mut state: ResMut<NextState<AppState>>,
) {
    if asteroids_query.is_empty() {
        level.increment();
        state.set(AppState::LoadLevel);
    }
}

fn gameover_system(
    ship_query: Query<&Ship>,
    mut state: ResMut<NextState<AppState>>,
    mut maybe_timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    let ship = ship_query.single();
    if ship.lives == 0 {
        if let Some(timer) = maybe_timer.as_mut() {
            if timer.tick(time.delta()).just_finished() {
                *maybe_timer = None;
                state.set(AppState::Title);
            }
        } else {
            *maybe_timer = Some(Timer::from_seconds(3.0, TimerMode::Once))
        }
    }
}

fn asteroid_score(size: AsteroidSize) -> u32 {
    match size {
        AsteroidSize::Tiny => 50,
        AsteroidSize::Small => 100,
        AsteroidSize::Medium => 150,
        AsteroidSize::Large => 200,
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
    mut ships_query: Query<(&mut Ship, &CollisionShape, &Transform)>,
    powerups_query: Query<(Entity, &Powerup, &CollisionShape)>,
    asset_server: Res<AssetServer>,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (mut ship, ship_shape, transform) in ships_query.iter_mut() {
        for (powerup_entity, powerup, powerup_shape) in powerups_query.iter() {
            if ship_shape.intersects(powerup_shape) {
                let text = match powerup {
                    Powerup::Laser => {
                        ship.weapon_rapid_level = (ship.weapon_rapid_level + 1).min(8);
                        "Laser +1"
                    }
                    Powerup::Spread => {
                        ship.weapon_spread_level = (ship.weapon_spread_level + 1).min(8);
                        "Spread +1"
                    }
                    Powerup::Beam => {
                        ship.weapon_beam_level = (ship.weapon_beam_level + 1).min(8);
                        "Beam +1"
                    }
                    Powerup::Plasma => {
                        ship.weapon_plasma_level = (ship.weapon_plasma_level + 1).min(8);
                        "Plasma +1"
                    }
                    Powerup::ExtraLife => {
                        ship.lives += 1;
                        "1up"
                    }
                    Powerup::LoseLife => {
                        ship.lives = ship.lives.max(1) - 1;
                        "-1up"
                    }
                    Powerup::Shield => {
                        ship.shield_level += 1;
                        "Shield +1"
                    }
                };
                commands.entity(powerup_entity).despawn();
                let position = transform.translation.truncate();
                commands.spawn(GameNotificationBundle::new(
                    text.to_owned(),
                    asset_server.load("fonts/DejaVuSans.ttf"),
                    position,
                    20.0,
                    1.0,
                ));
                commands.spawn(RingParticleBundle::new(position, &sprite_sheets.particles));
            }
        }
    }
}

fn ship_asteroid_collision_system(
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
    mut ships_query: Query<(&mut Ship, &Transform, &mut Moving, &CollisionShape)>,
    asteroids_query: Query<(&Transform, &Moving, &CollisionShape), (With<Asteroid>, Without<Ship>)>,
) {
    for (mut ship, ship_transform, mut ship_moving, ship_shape) in ships_query.iter_mut() {
        if ship.invulnerability > 0.0 {
            continue;
        }
        let ship_position = ship_transform.translation.truncate();
        for (asteroid_transform, asteroid_moving, asteroid_shape) in asteroids_query.iter() {
            let asteroid_position = asteroid_transform.translation.truncate();
            if ship_shape.intersects(asteroid_shape) {
                if ship.shield_level > 0 {
                    ship.shield_level -= 1;
                    let diff = (ship_position - asteroid_position).normalize();
                    let speed = asteroid_moving
                        .velocity
                        .project_onto_normalized(diff)
                        .length()
                        + ship_moving.velocity.project_onto_normalized(-diff).length();
                    ship_moving.velocity = diff * speed;
                } else {
                    ship.respawn_delay = SHIP_RESPAWN_DELAY;
                    ship.lives = ship.lives.max(1) - 1;
                    commands.spawn(ExplosionBundle::new(
                        &sprite_sheets.explosion,
                        ship_position,
                    ));
                    commands.spawn(WaveParticleBundle::new(
                        ship_position,
                        &sprite_sheets.particles,
                    ));
                }
            }
        }
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

fn collision_shape_system(mut query: Query<(&mut CollisionShape, &GlobalTransform)>) {
    for (mut shape, transform) in query.iter_mut() {
        shape.transform = transform.compute_transform();
    }
}

fn cheat_system(keyboard_input: Res<Input<KeyCode>>, mut ship_query: Query<&mut Ship>) {
    let mut ship = ship_query.single_mut();
    if keyboard_input.just_pressed(KeyCode::F1) {
        ship.weapon_rapid_level = ship.weapon_rapid_level.min(7) + 1;
    }
    if keyboard_input.just_pressed(KeyCode::F2) {
        ship.weapon_spread_level = ship.weapon_spread_level.min(7) + 1;
    }
    if keyboard_input.just_pressed(KeyCode::F3) {
        ship.weapon_beam_level = ship.weapon_beam_level.min(7) + 1;
    }
    if keyboard_input.just_pressed(KeyCode::F4) {
        ship.weapon_plasma_level = ship.weapon_plasma_level.min(7) + 1;
    }
    if keyboard_input.just_pressed(KeyCode::F5) {
        ship.shield_level += 1;
    }
    if keyboard_input.just_pressed(KeyCode::F6) {
        ship.lives += 1;
    }
}

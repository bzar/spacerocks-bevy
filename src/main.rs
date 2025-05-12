use std::f32::consts::TAU;

use bevy::{audio::Volume, prelude::*};
use rand::{Rng, random};

mod bundles;
mod components;
mod constants;
mod input;
mod plugins;
mod resources;
mod utils;

use crate::{components::*, constants::*, resources::*, utils::*};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Title,
    NewGame,
    LoadLevel,
    InGame,
    HighScore,
    HighScoreEntry,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(input::InputState::default())
        .insert_resource(SpriteSheets::default())
        .insert_resource(Level(0))
        .insert_resource(Score(0))
        .insert_resource(Sounds::default())
        .insert_resource(Mute { enabled: true })
        .insert_resource(LevelStartDelayTimer::default())
        .add_systems(Startup, init)
        .init_state::<AppState>()
        .add_plugins((
            plugins::CameraPlugin,
            plugins::TitleScreenPlugin,
            plugins::HighScorePlugin,
            plugins::MusicPlugin,
        ))
        .add_systems(
            Update,
            (
                input::update_input_state,
                spinning_system,
                wrapping_system,
                expiring_system,
                scaling_system,
                fading_system,
                animation_system,
                mute_system,
            ),
        )
        .add_systems(Update, loading.run_if(in_state(AppState::Loading)))
        .add_systems(OnEnter(AppState::NewGame), new_game)
        .add_systems(OnEnter(AppState::LoadLevel), load_level)
        .add_systems(
            Update,
            (
                level_start_delay_system,
                scaling_system,
                expiring_system,
                fading_system,
            )
                .run_if(in_state(AppState::LoadLevel)),
        )
        .add_systems(
            Update,
            (
                moving_system,
                ship_control_system,
                ship_physics,
                ship_sprite,
                ship_respawn_system,
                shield_sprite,
                beam_collision_shape_update_system,
                collision_shape_system,
                beam_sprite_system,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
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
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(OnExit(AppState::InGame), despawn_tagged::<LevelEntity>)
        .add_plugins((plugins::HudPlugin, plugins::UfoPlugin))
        .run();
}

fn despawn_tagged<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn init(asset_server: Res<AssetServer>, mut sprite_sheets: ResMut<SpriteSheets>) {
    sprite_sheets.load_state = asset_server.load_folder("img");
}
fn loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sprite_sheets: ResMut<SpriteSheets>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut loading_text: Local<Option<Entity>>,
    mut sounds: ResMut<Sounds>,
) {
    if loading_text.is_none() {
        let font = asset_server.load("fonts/DejaVuSans.ttf");
        *loading_text = Some(commands.spawn(bundles::loading_text(font)).id());
    }

    *sounds = Sounds {
        ship_explosion: asset_server.load("snd/sfx/explosion2.ogg"),
        rapid: asset_server.load("snd/sfx/weaponfire6.ogg"),
        spread: asset_server.load("snd/sfx/weaponfire5.ogg"),
        beam: asset_server.load("snd/sfx/weaponfire11-trimmed.ogg"),
        plasma: asset_server.load("snd/sfx/weaponfire4.ogg"),
        engine: asset_server.load("snd/sfx/enginehum3.ogg"),
        ufo_shoot: asset_server.load("snd/sfx/weaponfire9.ogg"),
        ufo_hit: asset_server.load("snd/sfx/weaponfire3.ogg"),
        ufo_explosion: asset_server.load("snd/sfx/explosion1.ogg"),
        title_1: asset_server.load("snd/sfx/weaponfire17.ogg"),
        title_2: asset_server.load("snd/sfx/weaponfire2.ogg"),
        title_3: asset_server.load("snd/sfx/explosion1.ogg"),
        powerup: asset_server.load("snd/sfx/weapon1probl.ogg"),
        extralife: asset_server.load("snd/sfx/game_showmenu.ogg"),
        loselife: asset_server.load("snd/sfx/game_hidemenu.ogg"),
        shield: asset_server.load("snd/sfx/antimaterhit.ogg"),
        asteroid_hit: asset_server.load("snd/sfx/explosion4.ogg"),
        asteroid_destroy: asset_server.load("snd/sfx/explosion2.ogg"),
        asteroid_destroy_small: asset_server.load("snd/sfx/explosion4.ogg"),
    };
    if asset_server
        .get_recursive_dependency_load_state(&sprite_sheets.load_state)
        .is_none_or(|state| state.is_loaded())
    {
        // Initialize texture atlases
        let asteroid_texture = asset_server.load("img/asteroids.png");
        let mut asteroid_atlas = TextureAtlasLayout::new_empty(UVec2::new(4, 5));

        fn asteroid_sprite_rects() -> impl Iterator<Item = URect> {
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
                    .map(move |(size_x, size)| URect {
                        min: UVec2::new(variant_x + size_x, variant_y),
                        max: UVec2::new(variant_x + size_x + size - 1, variant_y + size - 1),
                    })
            })
        }

        for asteroid_rect in asteroid_sprite_rects() {
            asteroid_atlas.add_texture(asteroid_rect);
        }

        sprite_sheets.asteroids_atlas = texture_atlases.add(asteroid_atlas);
        sprite_sheets.asteroids = asteroid_texture;

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
        commands.entity(ship_entity).despawn();
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
    sounds: Res<Sounds>,
) {
    println!("setup level {}", level.number());

    let asteroid_variant = level.asteroid_variant();

    let background_texture =
        asset_server.load(&format!("img/background-{}.png", level.background_image()));
    commands
        .spawn((
            Sprite::from_image(background_texture),
            Transform::from_xyz(0.0, 0.0, -0.09),
        ))
        .insert(LevelEntity);

    let mut rng = rand::rng();
    for size in level.asteroids() {
        let distance: f32 = rng.random_range(level.asteroid_distance_bounds());
        let direction = random::<f32>() * std::f32::consts::TAU;
        let position: Vec2 = Vec2::from_angle(direction) * distance;
        let heading = random::<f32>() * std::f32::consts::TAU;
        let speed = rng.random_range(level.asteroid_speed_bounds());
        let velocity = Vec2::from_angle(heading) * speed;
        let spinning_speed = random::<f32>() - 0.5;
        commands.spawn(bundles::asteroid(
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
        let beam_projectile = ShipProjectile::Beam;
        let beam_from = Vec2::ZERO;
        let beam_length = 0.0;
        let beam_max_length = 0.0;
        let beam_texture = asset_server.load("img/continuous_beam.png");
        let mut beam_transform = Transform::from_xyz(0.0, 0.0, -0.01);
        beam_transform.scale.y = beam_length / 128.0;
        commands.spawn((
            bundles::ship(ship, sprite_sheets.as_ref(), &sounds),
            children![
                bundles::ship_shield(&sprite_sheets.ship),
                (
                    bundles::ship_beam(
                        beam_projectile,
                        beam_texture,
                        beam_transform,
                        beam_from,
                        beam_length,
                        beam_max_length,
                        &sounds
                    ),
                    children![(
                        Sprite::from_image(asset_server.load("img/continuous_tip.png")),
                        Transform::from_xyz(0.0, 128.0, 0.0),
                        BeamTip
                    )]
                )
            ],
        ));
    } else {
        for (mut transform, mut moving) in ships_query.iter_mut() {
            transform.translation = Vec3::ZERO;
            moving.velocity = Vec2::ZERO;
            moving.acceleration = Vec2::ZERO;
        }
    }

    commands.spawn(bundles::game_notification(
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
        scaling.elapsed += time.delta_secs();
        let scale = lerp(scaling.from, scaling.to, scaling.elapsed / scaling.duration);
        transform.scale = Vec3::splat(scale);
    }
}
fn fading_system(
    mut fading_query: Query<(&mut Fading, Option<&mut TextColor>, Option<&mut Sprite>)>,
    time: Res<Time>,
) {
    for (mut fading, text_color, sprite) in fading_query.iter_mut() {
        fading.elapsed += time.delta_secs();
        let alpha = lerp(fading.from, fading.to, fading.elapsed / fading.duration);
        if let Some(mut text_color) = text_color {
            text_color.set_alpha(alpha);
        }
        if let Some(mut sprite) = sprite {
            sprite.color.set_alpha(alpha);
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
            commands.entity(entity).despawn()
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
            ship.respawn_delay -= time.delta_secs();
            if ship.respawn_delay > 0.0 {
                *visibility = Visibility::Hidden;
                ship.invulnerability = 100.0;
            } else {
                *visibility = Visibility::Visible;
                ship.invulnerability = SHIP_INVULNERABILITY;
                transform.translation = Vec3::ZERO;
                moving.velocity = Vec2::ZERO;
            }
        } else if ship.lives <= 0 {
            *visibility = Visibility::Hidden;
        }
    }
}
fn ship_control_system(mut ship_query: Query<&mut Ship>, input: Res<input::InputState>) {
    for mut ship in ship_query.iter_mut() {
        if ship.respawn_delay > 0.0 {
            ship.fire = false;
            continue;
        }
        ship.throttle = input.throttle;
        ship.turn = match (input.left, input.right) {
            (true, false) => ShipTurn::Left,
            (false, true) => ShipTurn::Right,
            _ => ShipTurn::Neutral,
        };
        ship.fire = input.fire;
        if input.weapon_1 && ship.weapon_rapid_level > 0 {
            ship.weapon = ShipWeapon::Rapid;
        } else if input.weapon_2 && ship.weapon_spread_level > 0 {
            ship.weapon = ShipWeapon::Spread;
        } else if input.weapon_3 && ship.weapon_beam_level > 0 {
            ship.weapon = ShipWeapon::Beam;
        } else if input.weapon_4 && ship.weapon_plasma_level > 0 {
            ship.weapon = ShipWeapon::Plasma;
        }

        if input.weapon_next {
            ship.next_weapon();
        }
        if input.weapon_prev {
            ship.prev_weapon();
        }
    }
}

fn ship_physics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ship_query: Query<(&mut Ship, &mut Moving, &mut Transform, &AudioSink)>,
    mut beam_query: Query<(&mut Beam, &AudioSink), Without<Ship>>,
    time: Res<Time>,
    sounds: Res<Sounds>,
) {
    let time_delta = time.delta().as_secs_f32();

    for (mut ship, mut moving, mut transform, engine_hum) in ship_query.iter_mut() {
        ship.invulnerability = (ship.invulnerability - time_delta).max(0.);
        let angular_velocity = match ship.turn {
            ShipTurn::Neutral => 0.0,
            ShipTurn::Left => 3.0,
            ShipTurn::Right => -3.0,
        };
        if ship.throttle && engine_hum.is_paused() {
            engine_hum.play();
        } else if !ship.throttle && !engine_hum.is_paused() {
            engine_hum.pause();
        }
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
                    commands.spawn(bundles::ship_projectile(
                        projectile,
                        texture.clone(),
                        velocity.clone(),
                        left_transform,
                        0.25,
                        1.0,
                    ));
                    commands.spawn(bundles::ship_projectile(
                        projectile,
                        texture,
                        velocity,
                        right_transform,
                        0.25,
                        1.0,
                    ));
                    commands.spawn(bundles::sfx(sounds.rapid.clone()));
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
                        commands.spawn(bundles::ship_projectile(
                            projectile,
                            texture.clone(),
                            velocity,
                            transform,
                            0.20,
                            1.0,
                        ));
                        commands.spawn(bundles::sfx(sounds.spread.clone()));
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
                    commands.spawn(bundles::ship_projectile(
                        projectile, texture, velocity, transform, 0.5, power,
                    ));
                    commands.spawn(bundles::sfx(sounds.plasma.clone()));
                    ship.weapon_cooldown =
                        lerp(1.2, 0.8, (ship.weapon_plasma_level - 1) as f32 / 8.0);
                }
                ShipWeapon::Beam => {
                    for (mut beam, sound) in beam_query.iter_mut() {
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
                        sound.play()
                    }
                }
            }
        } else if matches!(ship.weapon, ShipWeapon::Beam) {
            for (mut beam, sound) in beam_query.iter_mut() {
                sound.pause();
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

fn beam_collision_shape_update_system(mut query: Query<(&Beam, &mut CollisionShape)>) {
    for (beam, mut collision_shape) in query.iter_mut() {
        match collision_shape.shape {
            Shape::Line { ref mut delta, .. } => *delta = Vec2::Y * beam.length,
            _ => unimplemented!(),
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
            if let Ok(mut tip_transform) = tip_query.get_mut(child) {
                tip_transform.scale.y = 1.0 / transform.scale.y;
            }
        }
    }
}
fn ship_sprite(mut ship_query: Query<(&Ship, &mut Sprite)>, sprite_sheets: Res<SpriteSheets>) {
    for (ship, mut sprite) in ship_query.iter_mut() {
        sprite.image = sprite_sheets.ship.choose(&ship);
        let alpha = if ship.invulnerability > 0.0 { 0.5 } else { 1.0 };
        sprite.color.set_alpha(alpha);
    }
}

fn shield_sprite(
    ship_query: Query<&Ship>,
    mut ship_shield_query: Query<(&mut Visibility, &ChildOf), With<ShipShield>>,
) {
    for (mut visibility, child_of) in ship_shield_query.iter_mut() {
        let ship = ship_query.get(child_of.0);
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
    sounds: Res<Sounds>,
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
                        commands.spawn(bundles::sfx(sounds.asteroid_hit.clone()));
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
                        commands.spawn(bundles::sfx(sounds.asteroid_hit.clone()));
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
                                    commands.spawn(bundles::sfx(sounds.asteroid_hit.clone()));
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
                    commands.spawn(bundles::spark_particle(
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
    while let Some(
        [
            (mut a_moving, a_shape, a_transform),
            (mut b_moving, b_shape, b_transform),
        ],
    ) = pairs.fetch_next()
    {
        if a_shape.intersects(b_shape) {
            let a_position = a_transform.translation.truncate();
            let b_position = b_transform.translation.truncate();
            let diff = a_position - b_position;
            let epsilon = (a_moving.velocity - b_moving.velocity) * 0.01;
            if diff.length_squared() >= (diff + epsilon).length_squared() {
                let direction = diff.normalize();
                a_moving.velocity = direction * a_moving.velocity.length();
                b_moving.velocity = -direction * b_moving.velocity.length();
            }
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
    sounds: Res<Sounds>,
) {
    for (asteroid_entity, asteroid, transform) in asteroids.iter() {
        if asteroid.integrity <= 0 {
            let score_delta = asteroid_score(asteroid.size);
            score.increase(score_delta);
            commands.spawn(bundles::game_notification(
                format!("{}", score_delta),
                asset_server.load("fonts/DejaVuSans.ttf"),
                transform.translation.truncate(),
                20.0,
                1.0,
            ));
            commands.spawn(bundles::corona_particle(
                transform.translation.truncate(),
                asteroid.size.radius() / AsteroidSize::Large.radius(),
                &sprite_sheets.particles,
            ));
            if asteroid.size >= AsteroidSize::Medium {
                commands.spawn(bundles::sfx(sounds.asteroid_destroy.clone()));
            } else {
                commands.spawn(bundles::sfx(sounds.asteroid_destroy_small.clone()));
            }
            commands.entity(asteroid_entity).despawn();
            if let Some(size) = asteroid.size.smaller() {
                let direction = (transform.rotation * transform.translation)
                    .truncate()
                    .normalize();
                let n = level.asteroid_frag_count();
                let data = (0..n)
                    .map(|i| i as f32 * TAU / n as f32)
                    .map(|angle| direction.rotate(Vec2::from_angle(angle)));

                let parent_position = transform.translation.truncate();
                let spinning_speed = random::<f32>() - 0.5;
                for dir in data {
                    let position = parent_position + dir * 5.0;
                    let velocity = dir * 30.0;
                    commands.spawn(bundles::asteroid(
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
    if let Ok(ship) = ship_query.single() {
        if ship.lives == 0 {
            if let Some(timer) = maybe_timer.as_mut() {
                if timer.tick(time.delta()).just_finished() {
                    *maybe_timer = None;
                    state.set(AppState::HighScoreEntry);
                }
            } else {
                *maybe_timer = Some(Timer::from_seconds(3.0, TimerMode::Once))
            }
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

impl rand::distr::Distribution<Powerup> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Powerup {
        use Powerup::*;
        match rng.random_range(0..7) {
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
    sounds: Res<Sounds>,
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

                match powerup {
                    Powerup::Laser | Powerup::Spread | Powerup::Beam | Powerup::Plasma => {
                        commands.spawn(bundles::sfx(sounds.powerup.clone()));
                    }
                    Powerup::ExtraLife => {
                        commands.spawn(bundles::sfx(sounds.extralife.clone()));
                    }
                    Powerup::LoseLife => {
                        commands.spawn(bundles::sfx(sounds.loselife.clone()));
                    }
                    Powerup::Shield => {
                        commands.spawn(bundles::sfx(sounds.shield.clone()));
                    }
                };
                commands.entity(powerup_entity).despawn();
                let position = transform.translation.truncate();
                commands.spawn(bundles::game_notification(
                    text.to_owned(),
                    asset_server.load("fonts/DejaVuSans.ttf"),
                    position,
                    20.0,
                    1.0,
                ));
                commands.spawn(bundles::ring_particle(position, &sprite_sheets.particles));
            }
        }
    }
}

fn ship_asteroid_collision_system(
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
    mut ships_query: Query<(&mut Ship, &Transform, &mut Moving, &CollisionShape)>,
    asteroids_query: Query<(&Transform, &Moving, &CollisionShape), (With<Asteroid>, Without<Ship>)>,
    sounds: Res<Sounds>,
) {
    for (mut ship, ship_transform, mut ship_moving, ship_shape) in ships_query.iter_mut() {
        if ship.invulnerability > 0.0 {
            continue;
        }
        let ship_position = ship_transform.translation.truncate();
        for (asteroid_transform, asteroid_moving, asteroid_shape) in asteroids_query.iter() {
            let asteroid_position = asteroid_transform.translation.truncate();
            if ship_shape.intersects(asteroid_shape) {
                let diff = ship_position - asteroid_position;
                let epsilon = (ship_moving.velocity - asteroid_moving.velocity) * 0.01;
                if diff.length_squared() < (diff + epsilon).length_squared() {
                    continue;
                }
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
                    ship.die();
                    commands.spawn(bundles::explosion(&sprite_sheets.explosion, ship_position));
                    commands.spawn(bundles::wave_particle(
                        ship_position,
                        &sprite_sheets.particles,
                    ));
                    commands.spawn(bundles::sfx(sounds.ship_explosion.clone()));
                }
            }
        }
    }
}

fn animation_system(mut animated_query: Query<(&mut Animated, &mut Sprite)>, time: Res<Time>) {
    let delta = time.delta_secs();
    for (mut animated, mut sprite) in animated_query.iter_mut() {
        animated.elapsed += delta;
        let position = if animated.looping {
            animated.elapsed.rem_euclid(animated.animation.duration)
        } else {
            animated.elapsed.min(animated.animation.duration)
        };
        let frame = ((animated.animation.frames.len() - 1) as f32 * position
            / animated.animation.duration)
            .floor() as usize;

        sprite.image = animated.animation.frames[frame].clone()
    }
}

fn collision_shape_system(mut query: Query<(&mut CollisionShape, &GlobalTransform)>) {
    for (mut shape, transform) in query.iter_mut() {
        shape.transform = transform.compute_transform();
    }
}

fn cheat_system(keyboard_input: Res<ButtonInput<KeyCode>>, mut ship_query: Query<&mut Ship>) {
    if let Ok(mut ship) = ship_query.single_mut() {
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
}

fn mute_system(
    mut sinks: Query<&mut AudioSink>,
    mute: Res<resources::Mute>,
    mut global_volume: ResMut<GlobalVolume>,
) {
    if mute.enabled {
        global_volume.volume = Volume::SILENT;
        for mut sink in sinks.iter_mut() {
            sink.set_volume(Volume::SILENT);
        }
    } else {
        global_volume.volume = Volume::Linear(1.0);
        for mut sink in sinks.iter_mut() {
            sink.set_volume(Volume::Linear(1.0));
        }
    }
}

use crate::{components::*, constants::*, resources::*};
use bevy::prelude::*;

#[derive(Bundle)]
pub struct PowerupBundle {
    pub sprite_bundle: SpriteBundle,
    pub powerup: Powerup,
    pub moving: Moving,
    pub collision_shape: CollisionShape,
    pub expiring: Expiring,
    pub wrapping: Wrapping,
    pub level_entity: LevelEntity,
}

impl PowerupBundle {
    pub fn new(
        powerup: Powerup,
        position: Vec2,
        velocity: Vec2,
        life: f32,
        sprite_sheet: &PowerupImages,
    ) -> Self {
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
        let transform = Transform::from_translation(position.extend(-0.01));
        Self {
            sprite_bundle: SpriteBundle {
                texture,
                transform,
                ..Default::default()
            },
            powerup,
            moving: Moving {
                velocity,
                acceleration: Vec2::ZERO,
            },
            collision_shape: CollisionShape::new(
                Shape::Circle {
                    center: Vec2::ZERO,
                    radius: 16.0,
                },
                transform,
            ),
            expiring: Expiring { life },
            wrapping: Wrapping,
            level_entity: LevelEntity,
        }
    }
}

#[derive(Bundle)]
pub struct LoadingTextBundle {
    text_bundle: TextBundle,
}
impl LoadingTextBundle {
    pub fn new(font: Handle<Font>) -> Self {
        let text_bundle = TextBundle {
            text: Text::from_section(
                "Loading...",
                TextStyle {
                    font,
                    font_size: 100.0,
                    color: Color::WHITE,
                },
            ),
            style: Style {
                ..Default::default()
            },
            ..Default::default()
        };
        LoadingTextBundle { text_bundle }
    }
}

fn asteroid_texture_index(variant: usize, size: AsteroidSize) -> usize {
    variant * ASTEROID_SIZES + size as usize
}

#[derive(Bundle)]
pub struct AsteroidBundle {
    sprite_sheet_bundle: SpriteSheetBundle,
    moving: Moving,
    spinning: Spinning,
    wrapping: Wrapping,
    asteroid: Asteroid,
    level_entity: LevelEntity,
    collision_shape: CollisionShape,
}
impl AsteroidBundle {
    pub fn new(
        sprite_sheets: &SpriteSheets,
        asteroid_variant: usize,
        size: AsteroidSize,
        position: Vec2,
        velocity: Vec2,
        spinning_speed: f32,
    ) -> Self {
        let sprite_sheet_bundle = SpriteSheetBundle {
            texture_atlas: sprite_sheets.asteroids.clone(),
            sprite: TextureAtlasSprite::new(asteroid_texture_index(asteroid_variant, size)),
            transform: Transform::from_translation(position.extend(0.)),
            ..Default::default()
        };
        let moving = Moving {
            velocity,
            ..Default::default()
        };
        let spinning = Spinning {
            speed: spinning_speed,
        };
        let asteroid = Asteroid {
            size,
            integrity: size as i32 * 4 + 1,
            variant: asteroid_variant,
        };
        AsteroidBundle {
            sprite_sheet_bundle,
            moving,
            spinning,
            asteroid,
            wrapping: Wrapping,
            level_entity: LevelEntity,
            collision_shape: CollisionShape::new(
                Shape::Circle {
                    center: Vec2::ZERO,
                    radius: size.radius(),
                },
                Transform::from_translation(position.extend(0.)),
            ),
        }
    }
}

#[derive(Bundle)]
pub struct ShipBundle {
    sprite_bundle: SpriteBundle,
    moving: Moving,
    wrapping: Wrapping,
    ship: Ship,
    collision_shape: CollisionShape,
}
impl ShipBundle {
    pub fn new(ship: Ship, sprite_sheets: &SpriteSheets) -> Self {
        let sprite_bundle = SpriteBundle {
            texture: sprite_sheets.ship.choose(&ship),
            ..Default::default()
        };
        ShipBundle {
            sprite_bundle,
            moving: Moving::default(),
            wrapping: Wrapping,
            ship,
            collision_shape: CollisionShape::new(
                Shape::Circle {
                    center: Vec2::ZERO,
                    radius: 12.0,
                },
                Transform::default(),
            ),
        }
    }
}

#[derive(Bundle)]
pub struct ShipShieldBundle {
    sprite_bundle: SpriteBundle,
    ship_shield: ShipShield,
}
impl ShipShieldBundle {
    pub fn new(ship_images: &ShipImages) -> Self {
        ShipShieldBundle {
            sprite_bundle: SpriteBundle {
                visibility: Visibility::Hidden,
                texture: ship_images.shield.clone(),
                ..Default::default()
            },
            ship_shield: ShipShield,
        }
    }
}

#[derive(Bundle)]
pub struct ShipProjectileBundle {
    sprite_bundle: SpriteBundle,
    moving: Moving,
    wrapping: Wrapping,
    ship_projectile: ShipProjectile,
    expiring: Expiring,
    collision_shape: CollisionShape,
}
impl ShipProjectileBundle {
    pub fn new(
        ship_projectile: ShipProjectile,
        texture: Handle<Image>,
        velocity: Vec2,
        transform: Transform,
        life: f32,
        radius: f32,
    ) -> Self {
        ShipProjectileBundle {
            sprite_bundle: SpriteBundle {
                texture,
                transform,
                ..Default::default()
            },
            moving: Moving {
                velocity,
                ..Default::default()
            },
            wrapping: Wrapping,
            ship_projectile,
            expiring: Expiring { life },
            collision_shape: CollisionShape::new(
                Shape::Circle {
                    center: Vec2::ZERO,
                    radius,
                },
                transform,
            ),
        }
    }
}

#[derive(Bundle)]
pub struct ExplosionBundle {
    sprite_bundle: SpriteBundle,
    animated: Animated,
    expiring: Expiring,
}
impl ExplosionBundle {
    pub fn new(explosion_images: &ExplosionImages, position: Vec2) -> ExplosionBundle {
        ExplosionBundle {
            sprite_bundle: SpriteBundle {
                texture: explosion_images.normal[0].clone(),
                transform: Transform::from_translation(position.extend(0.09)),
                ..Default::default()
            },
            animated: Animated {
                animation: Animation {
                    frames: explosion_images.normal.clone(),
                    duration: 2.0,
                },
                elapsed: 0.0,
                looping: false,
            },
            expiring: Expiring { life: 2.0 },
        }
    }
}

#[derive(Bundle)]
pub struct GameNotificationBundle {
    text_2d_bundle: Text2dBundle,
    scaling: Scaling,
    fading: Fading,
    expiring: Expiring,
}
impl GameNotificationBundle {
    pub fn new(
        text: String,
        font: Handle<Font>,
        position: Vec2,
        size: f32,
        duration: f32,
    ) -> GameNotificationBundle {
        GameNotificationBundle {
            text_2d_bundle: Text2dBundle {
                text: Text::from_section(
                    text,
                    TextStyle {
                        font,
                        font_size: size,
                        color: Color::WHITE,
                    },
                ),
                transform: Transform::from_translation(position.extend(0.1)),
                ..Default::default()
            },
            scaling: Scaling {
                from: 1.0,
                to: 2.0,
                duration,
                elapsed: 0.0,
            },
            fading: Fading {
                from: 0.5,
                to: 0.0,
                duration,
                elapsed: 0.0,
            },
            expiring: Expiring { life: duration },
        }
    }
}

#[derive(Bundle)]
pub struct WaveParticleBundle {
    sprite_bundle: SpriteBundle,
    expiring: Expiring,
    scaling: Scaling,
    fading: Fading,
}
impl WaveParticleBundle {
    pub fn new(position: Vec2, particle_images: &ParticleImages) -> WaveParticleBundle {
        WaveParticleBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position.extend(0.),
                    scale: Vec3::splat(0.0),
                    ..Default::default()
                },
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.1),
                    ..Default::default()
                },
                texture: particle_images.wave.clone(),
                ..Default::default()
            },
            expiring: Expiring { life: 1.0 },
            scaling: Scaling {
                from: 0.0,
                to: 1.0,
                duration: 1.0,
                elapsed: 0.0,
            },
            fading: Fading {
                from: 0.1,
                to: 0.0,
                duration: 1.0,
                elapsed: 0.0,
            },
        }
    }
}

#[derive(Bundle)]
pub struct RingParticleBundle {
    sprite_bundle: SpriteBundle,
    expiring: Expiring,
    scaling: Scaling,
    fading: Fading,
}
impl RingParticleBundle {
    pub fn new(position: Vec2, particle_images: &ParticleImages) -> RingParticleBundle {
        RingParticleBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position.extend(0.),
                    scale: Vec3::splat(0.0),
                    ..Default::default()
                },
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.1),
                    ..Default::default()
                },
                texture: particle_images.ring.clone(),
                ..Default::default()
            },
            expiring: Expiring { life: 1.0 },
            scaling: Scaling {
                from: 0.0,
                to: 1.0,
                duration: 1.0,
                elapsed: 0.0,
            },
            fading: Fading {
                from: 0.2,
                to: 0.0,
                duration: 1.0,
                elapsed: 0.0,
            },
        }
    }
}
#[derive(Bundle)]
pub struct CoronaParticleBundle {
    sprite_bundle: SpriteBundle,
    expiring: Expiring,
    fading: Fading,
}
impl CoronaParticleBundle {
    pub fn new(
        position: Vec2,
        size: f32,
        particle_images: &ParticleImages,
    ) -> CoronaParticleBundle {
        CoronaParticleBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position.extend(0.),
                    scale: Vec3::splat(size),
                    ..Default::default()
                },
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.1),
                    ..Default::default()
                },
                texture: particle_images.corona.clone(),
                ..Default::default()
            },
            expiring: Expiring { life: 1.0 },
            fading: Fading {
                from: 0.1,
                to: 0.0,
                duration: 1.0,
                elapsed: 0.0,
            },
        }
    }
}
#[derive(Bundle)]
pub struct SparkParticleBundle {
    sprite_bundle: SpriteBundle,
    moving: Moving,
    expiring: Expiring,
    scaling: Scaling,
    spinning: Spinning,
    fading: Fading,
}
impl SparkParticleBundle {
    pub fn new(
        position: Vec2,
        velocity: Vec2,
        acceleration: Vec2,
        particle_images: &ParticleImages,
    ) -> SparkParticleBundle {
        SparkParticleBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position.extend(0.),
                    scale: Vec3::splat(0.0),
                    ..Default::default()
                },
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.1),
                    ..Default::default()
                },
                texture: particle_images.spark.clone(),
                ..Default::default()
            },
            moving: Moving {
                velocity,
                acceleration,
            },
            expiring: Expiring { life: 1.0 },
            scaling: Scaling {
                from: 0.0,
                to: 1.0,
                duration: 1.0,
                elapsed: 0.0,
            },
            spinning: Spinning { speed: 1.0 },
            fading: Fading {
                from: 1.0,
                to: 0.0,
                duration: 1.0,
                elapsed: 0.0,
            },
        }
    }
}

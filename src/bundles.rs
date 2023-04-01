use crate::{components::*, constants::*, resources::*};
use bevy::prelude::*;

#[derive(Bundle)]
pub struct PowerupBundle {
    pub sprite_bundle: SpriteBundle,
    pub powerup: Powerup,
    pub moving: Moving,
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
            collision_shape: CollisionShape::Circle {
                center: position,
                radius: size.radius(),
            },
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
    pub fn new(sprite_sheets: &SpriteSheets) -> Self {
        let ship = Ship {
            weapon_rapid_level: 4,
            weapon_spread_level: 4,
            weapon_beam_level: 4,
            weapon_plasma_level: 8,
            shield_level: 2,
            ..Ship::default()
        };
        let sprite_bundle = SpriteBundle {
            texture: sprite_sheets.ship.choose(&ship),
            ..Default::default()
        };
        ShipBundle {
            sprite_bundle,
            moving: Moving::default(),
            wrapping: Wrapping,
            ship,
            collision_shape: CollisionShape::Circle {
                center: Vec2::ZERO,
                radius: 12.0,
            },
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
            collision_shape: CollisionShape::Circle {
                center: transform.translation.truncate(),
                radius,
            },
        }
    }
}

#[derive(Bundle)]
pub struct UfoBundle {
    sprite_bundle: SpriteBundle,
    ufo: Ufo,
    level_entity: LevelEntity,
    collision_shape: CollisionShape,
}
impl UfoBundle {
    pub fn new(ufo_images: &UfoImages, ufo: Ufo) -> Self {
        let center = ufo.start_position.clone();
        UfoBundle {
            sprite_bundle: SpriteBundle {
                texture: ufo_images.ship[0].clone(),
                transform: Transform::from_translation(ufo.start_position.extend(0.)),
                ..Default::default()
            },
            ufo,
            level_entity: LevelEntity,
            collision_shape: CollisionShape::Circle {
                center,
                radius: 16.0,
            },
        }
    }
}

#[derive(Bundle)]
pub struct UfoLaserBundle {
    sprite_bundle: SpriteBundle,
    ufo_laser: UfoLaser,
    moving: Moving,
    expiring: Expiring,
    collision_shape: CollisionShape,
}
impl UfoLaserBundle {
    pub fn new(
        ufo_images: &UfoImages,
        position: Vec2,
        rotation: f32,
        velocity: Vec2,
        life: f32,
    ) -> Self {
        UfoLaserBundle {
            sprite_bundle: SpriteBundle {
                texture: ufo_images.laser.clone(),
                transform: Transform::from_translation(position.extend(0.))
                    .with_rotation(Quat::from_rotation_z(rotation)),
                ..Default::default()
            },
            ufo_laser: UfoLaser,
            moving: Moving {
                velocity,
                ..Default::default()
            },
            expiring: Expiring { life },
            collision_shape: CollisionShape::Circle {
                center: position,
                radius: 1.0,
            },
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

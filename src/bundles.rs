use crate::{components::*, constants::*, resources::*};
use bevy::{
    prelude::*,
    sprite::{Anchor, SpriteSystem},
};

pub fn powerup(
    powerup: Powerup,
    position: Vec2,
    velocity: Vec2,
    life: f32,
    sprite_sheet: &PowerupImages,
) -> impl Bundle {
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
    (
        Sprite::from_image(texture),
        transform,
        powerup,
        Moving::from_velocity(velocity),
        CollisionShape::new(
            Shape::Circle {
                center: Vec2::ZERO,
                radius: 16.0,
            },
            transform,
        ),
        Expiring { life },
        Wrapping,
        LevelEntity,
    )
}

pub fn loading_text(font: Handle<Font>) -> impl Bundle {
    (
        Text2d("Loading...".to_owned()),
        TextFont::from_font(font).with_font_size(100.0),
        TextColor::WHITE,
    )
}

fn asteroid_texture_index(variant: usize, size: AsteroidSize) -> usize {
    variant * ASTEROID_SIZES + size as usize
}

pub fn asteroid(
    sprite_sheets: &SpriteSheets,
    asteroid_variant: usize,
    size: AsteroidSize,
    position: Vec2,
    velocity: Vec2,
    spinning_speed: f32,
) -> impl Bundle {
    (
        Sprite::from_atlas_image(
            sprite_sheets.asteroids.clone(),
            TextureAtlas {
                layout: sprite_sheets.asteroids_atlas.clone(),
                index: asteroid_texture_index(asteroid_variant, size),
            },
        ),
        Transform::from_translation(position.extend(0.1)),
        Moving::from_velocity(velocity),
        Spinning {
            speed: spinning_speed,
        },
        Asteroid {
            size,
            integrity: size as i32 * 4 + 1,
            variant: asteroid_variant,
        },
        Wrapping,
        LevelEntity,
        CollisionShape::new(
            Shape::Circle {
                center: Vec2::ZERO,
                radius: size.radius(),
            },
            Transform::from_translation(position.extend(0.)),
        ),
    )
}

pub fn ship(ship: Ship, sprite_sheets: &SpriteSheets) -> impl Bundle {
    (
        Sprite::from_image(sprite_sheets.ship.choose(&ship)),
        Moving::default(),
        Wrapping,
        ship,
        CollisionShape::new(
            Shape::Circle {
                center: Vec2::ZERO,
                radius: 12.0,
            },
            Transform::default(),
        ),
    )
}

pub fn ship_shield(ship_images: &ShipImages) -> impl Bundle {
    (
        Sprite::from_image(ship_images.shield.clone()),
        Visibility::Hidden,
        ShipShield,
    )
}

pub fn ship_projectile(
    ship_projectile: ShipProjectile,
    texture: Handle<Image>,
    velocity: Vec2,
    transform: Transform,
    life: f32,
    radius: f32,
) -> impl Bundle {
    (
        Sprite::from_image(texture),
        transform,
        Moving::from_velocity(velocity),
        Wrapping,
        ship_projectile,
        Expiring { life },
        CollisionShape::new(
            Shape::Circle {
                center: Vec2::ZERO,
                radius,
            },
            transform,
        ),
    )
}

pub fn ship_beam(
    ship_projectile: ShipProjectile,
    texture: Handle<Image>,
    transform: Transform,
    base: Vec2,
    length: f32,
    max_length: f32,
) -> impl Bundle {
    (
        Sprite {
            image: texture,
            anchor: Anchor::BottomCenter,
            ..Default::default()
        },
        transform,
        Beam {
            length,
            max_length,
            sustained: 0.0,
            cooldown: 0.0,
            active: true,
        },
        ship_projectile,
        CollisionShape::new(
            Shape::Line {
                base: Vec2::ZERO,
                delta: Vec2::Y * 128.0,
                width: 8.0,
            },
            Transform::from_translation(base.extend(0.)),
        ),
    )
}

pub fn explosion(explosion_images: &ExplosionImages, position: Vec2) -> impl Bundle {
    (
        Sprite::from_image(explosion_images.normal[0].clone()),
        Transform::from_translation(position.extend(0.09)),
        Animated {
            animation: Animation {
                frames: explosion_images.normal.clone(),
                duration: 2.0,
            },
            elapsed: 0.0,
            looping: false,
        },
        Expiring { life: 2.0 },
    )
}

pub fn game_notification(
    text: String,
    font: Handle<Font>,
    position: Vec2,
    size: f32,
    duration: f32,
) -> impl Bundle {
    (
        Text2d::new(text),
        TextFont::from_font(font).with_font_size(size),
        TextColor::WHITE,
        Transform::from_translation(position.extend(0.1)),
        Scaling {
            from: 1.0,
            to: 2.0,
            duration,
            elapsed: 0.0,
        },
        Fading {
            from: 0.5,
            to: 0.0,
            duration,
            elapsed: 0.0,
        },
        Expiring { life: duration },
    )
}

pub fn wave_particle(position: Vec2, particle_images: &ParticleImages) -> impl Bundle {
    (
        Sprite {
            image: particle_images.wave.clone(),
            color: Color::srgba(1.0, 1.0, 1.0, 0.1),
            ..Default::default()
        },
        Transform {
            translation: position.extend(0.),
            scale: Vec3::splat(0.0),
            ..Default::default()
        },
        Expiring { life: 1.0 },
        Scaling {
            from: 0.0,
            to: 1.0,
            duration: 1.0,
            elapsed: 0.0,
        },
        Fading {
            from: 0.1,
            to: 0.0,
            duration: 1.0,
            elapsed: 0.0,
        },
    )
}

pub fn ring_particle(position: Vec2, particle_images: &ParticleImages) -> impl Bundle {
    (
        Sprite {
            image: particle_images.ring.clone(),
            color: Color::srgba(1.0, 1.0, 1.0, 0.1),
            ..Default::default()
        },
        Transform {
            translation: position.extend(0.),
            scale: Vec3::splat(0.0),
            ..Default::default()
        },
        Expiring { life: 1.0 },
        Scaling {
            from: 0.0,
            to: 1.0,
            duration: 1.0,
            elapsed: 0.0,
        },
        Fading {
            from: 0.2,
            to: 0.0,
            duration: 1.0,
            elapsed: 0.0,
        },
    )
}

pub fn corona_particle(position: Vec2, size: f32, particle_images: &ParticleImages) -> impl Bundle {
    (
        Sprite {
            image: particle_images.corona.clone(),
            color: Color::srgba(1.0, 1.0, 1.0, 0.1),
            ..Default::default()
        },
        Transform {
            translation: position.extend(0.),
            scale: Vec3::splat(size),
            ..Default::default()
        },
        Expiring { life: 1.0 },
        Fading {
            from: 0.2,
            to: 0.0,
            duration: 1.0,
            elapsed: 0.0,
        },
    )
}

pub fn spark_particle(
    position: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
    particle_images: &ParticleImages,
) -> impl Bundle {
    (
        Sprite {
            image: particle_images.spark.clone(),
            color: Color::srgba(1.0, 1.0, 1.0, 0.1),
            ..Default::default()
        },
        Transform {
            translation: position.extend(0.),
            scale: Vec3::splat(0.0),
            ..Default::default()
        },
        Moving {
            velocity,
            acceleration,
        },
        Expiring { life: 1.0 },
        Scaling {
            from: 0.0,
            to: 1.0,
            duration: 1.0,
            elapsed: 0.0,
        },
        Spinning { speed: 1.0 },
        Fading {
            from: 1.0,
            to: 0.0,
            duration: 1.0,
            elapsed: 0.0,
        },
    )
}

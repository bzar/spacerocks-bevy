use bevy::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum AsteroidSize {
    Tiny = 0,
    Small,
    Medium,
    Large,
}

impl AsteroidSize {
    pub fn smaller(&self) -> Option<AsteroidSize> {
        match self {
            AsteroidSize::Tiny => None,
            AsteroidSize::Small => Some(AsteroidSize::Tiny),
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Large => Some(AsteroidSize::Medium),
        }
    }
    pub fn radius(&self) -> f32 {
        match self {
            AsteroidSize::Tiny => 4.0,
            AsteroidSize::Small => 8.0,
            AsteroidSize::Medium => 16.0,
            AsteroidSize::Large => 24.0,
        }
    }
    pub fn cost(&self) -> u32 {
        match self {
            AsteroidSize::Tiny => 1,
            _ => 2 * self.smaller().unwrap().cost(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ShipWeapon {
    Rapid,
    Spread,
    Beam,
    Plasma,
}

#[derive(Component, Clone, Copy)]
pub enum ShipProjectile {
    Rapid,
    Spread,
    Beam { power: f32 },
    Plasma { power: f32 },
}

#[derive(Component)]
pub struct Beam {
    pub length: f32,
    pub max_length: f32,
    pub sustained: f32,
    pub cooldown: f32,
}

#[derive(Clone, Copy)]
pub enum ShipTurn {
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
pub struct Expiring {
    pub life: f32,
}

impl Default for ShipWeapon {
    fn default() -> Self {
        Self::Rapid
    }
}

#[derive(Component)]
pub struct Asteroid {
    pub size: AsteroidSize,
    pub integrity: i32,
    pub variant: usize,
}

#[derive(Component)]
pub struct Ufo {
    pub start_position: Vec2,
    pub end_position: Vec2,
    pub frequency: f32,
    pub amplitude: f32,
    pub duration: f32,
    pub time: f32,
    pub shoot_delay: f32,
    pub shoot_accuracy: f32,
    pub life: i32,
}
#[derive(Component)]
pub struct UfoLaser;

#[derive(Component)]
pub enum Powerup {
    Laser = 0,
    Spread,
    Beam,
    Plasma,
    ExtraLife,
    LoseLife,
    Shield,
}

#[derive(Component, Default)]
pub struct Moving {
    pub velocity: Vec2,
    pub acceleration: Vec2,
}

#[derive(Component)]
pub struct Spinning {
    pub speed: f32,
}

#[derive(Component)]
pub struct Wrapping;

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component, Default, PartialEq, Eq)]
pub struct HUD {
    pub level: u32,
    pub score: u32,
    pub lives: u8,
    pub weapon: ShipWeapon,
    pub weapon_rapid_level: u8,
    pub weapon_spread_level: u8,
    pub weapon_beam_level: u8,
    pub weapon_plasma_level: u8,
}

#[derive(Component, Default)]
pub struct Ship {
    pub throttle: bool,
    pub turn: ShipTurn,
    pub fire: bool,
    pub weapon: ShipWeapon,
    pub weapon_rapid_level: u8,
    pub weapon_spread_level: u8,
    pub weapon_beam_level: u8,
    pub weapon_plasma_level: u8,
    pub weapon_cooldown: f32,
    pub shield_level: u8,
    pub lives: u8,
    pub invulnerability: f32,
    pub respawn_delay: f32,
}

#[derive(Component)]
pub struct ShipShield;

pub struct Animation {
    pub frames: Vec<Handle<Image>>,
    pub duration: f32,
}

#[derive(Component)]
pub struct Animated {
    pub animation: Animation,
    pub elapsed: f32,
    pub looping: bool,
}

#[derive(Debug)]
pub enum Shape {
    Circle { center: Vec2, radius: f32 },
    Line { base: Vec2, delta: Vec2, width: f32 },
}

impl Shape {
    pub fn intersects(&self, other: &Shape) -> bool {
        use Shape::*;
        match (self, other) {
            (
                Circle {
                    center: c1,
                    radius: r1,
                },
                Circle {
                    center: c2,
                    radius: r2,
                },
            ) => c1.distance_squared(*c2) <= (r1 + r2).powi(2),
            (Circle { center, radius }, Line { base, delta, width })
            | (Line { base, delta, width }, Circle { center, radius }) => {
                let norm = delta.perp().normalize();
                let a = *center - *base;
                let b = *center - *base - *delta;
                if norm.perp_dot(a) * norm.perp_dot(b) < 0.0 {
                    a.project_onto(norm).length_squared() < (radius + width).powi(2)
                } else if a.length_squared() < b.length_squared() {
                    a.length_squared() <= (radius + width).powi(2)
                } else {
                    b.length_squared() <= (radius + width).powi(2)
                }
            }
            _ => unimplemented!(),
        }
    }
    pub fn distance(&self, other: &Shape) -> f32 {
        use Shape::*;
        match (self, other) {
            (
                Circle {
                    center: c1,
                    radius: r1,
                },
                Circle {
                    center: c2,
                    radius: r2,
                },
            ) => c1.distance(*c2) - r1 - r2,
            (Circle { center, radius }, Line { base, delta, width })
            | (Line { base, delta, width }, Circle { center, radius }) => {
                // Assumes previously verified intersection

                // Find the point halfway to the other side of the circle
                let l1q = (*center - *base).project_onto(*delta);
                let q = l1q + *base;
                // Use pythagorean theorem to find distance squared from halfway point to circle edge
                let s2 = (radius + width).powi(2) - (*center - q).length_squared();
                // Calculate relative distance from base to circle edge along l1q
                let t = 1.0 - s2 / l1q.length_squared();
                // Distance to edge
                (l1q * t).length()
            }
            _ => unimplemented!(),
        }
    }

    pub fn transformed(&self, transform: &Transform) -> Shape {
        use Shape::*;
        match self {
            Circle { center, radius } => Circle {
                center: *center + transform.translation.truncate(),
                radius: radius * transform.scale.max_element(), // TODO
            },
            Line { base, delta, width } => Line {
                base: *base + transform.translation.truncate(),
                delta: transform.rotation.mul_vec3(delta.extend(0.)).truncate(),
                width: width * transform.scale.max_element(), // TODO
            },
        }
    }
}
#[derive(Component)]
pub struct CollisionShape {
    pub shape: Shape,
    pub transform: Transform,
}

impl CollisionShape {
    pub fn new(shape: Shape, transform: Transform) -> Self {
        CollisionShape { shape, transform }
    }
    fn global_shape(&self) -> Shape {
        self.shape.transformed(&self.transform)
    }
    pub fn intersects(&self, other: &CollisionShape) -> bool {
        self.global_shape().intersects(&other.global_shape())
    }
    pub fn distance(&self, other: &CollisionShape) -> f32 {
        self.global_shape().distance(&other.global_shape())
    }
}

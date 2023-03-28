use crate::{components::*, constants::*, utils::*};
use bevy::prelude::*;

#[derive(Default)]
pub struct ShipImages {
    pub rapid: Handle<Image>,
    pub rapid_accelerating: Handle<Image>,
    pub rapid_left: Handle<Image>,
    pub rapid_left_accelerating: Handle<Image>,
    pub rapid_right: Handle<Image>,
    pub rapid_right_accelerating: Handle<Image>,
    pub spread: Handle<Image>,
    pub spread_accelerating: Handle<Image>,
    pub spread_left: Handle<Image>,
    pub spread_left_accelerating: Handle<Image>,
    pub spread_right: Handle<Image>,
    pub spread_right_accelerating: Handle<Image>,
    pub beam: Handle<Image>,
    pub beam_accelerating: Handle<Image>,
    pub beam_left: Handle<Image>,
    pub beam_left_accelerating: Handle<Image>,
    pub beam_right: Handle<Image>,
    pub beam_right_accelerating: Handle<Image>,
    pub plasma: Handle<Image>,
    pub plasma_accelerating: Handle<Image>,
    pub plasma_left: Handle<Image>,
    pub plasma_left_accelerating: Handle<Image>,
    pub plasma_right: Handle<Image>,
    pub plasma_right_accelerating: Handle<Image>,
    pub shield: Handle<Image>,
}

#[derive(Default)]
pub struct UfoImages {
    pub ship: Vec<Handle<Image>>,
    pub laser: Handle<Image>,
}

#[derive(Default)]
pub struct PowerupImages {
    pub laser: Handle<Image>,
    pub spread: Handle<Image>,
    pub beam: Handle<Image>,
    pub plasma: Handle<Image>,
    pub extra_life: Handle<Image>,
    pub lose_life: Handle<Image>,
    pub shield: Handle<Image>,
}

#[derive(Default, Resource)]
pub struct SpriteSheets {
    pub asteroids: Handle<TextureAtlas>,
    pub images: Vec<HandleUntyped>,
    pub ship: ShipImages,
    pub ufo: UfoImages,
    pub powerup: PowerupImages,
}

pub struct Level(pub u32);

#[derive(Resource)]
pub struct GameState {
    pub level: Level,
    pub score: u32,
    pub next_ufo_score: u32,
}

impl ShipImages {
    pub fn choose(&self, ship: &Ship) -> Handle<Image> {
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

impl Level {
    pub fn increment(&mut self) {
        self.0 += 1;
    }
    pub fn asteroid_variant(&self) -> usize {
        self.0 as usize % ASTEROID_VARIANTS
    }
    pub fn background_image(&self) -> usize {
        self.0 as usize % BACKGROUND_IMAGES + 1
    }
    pub fn asteroids(&self) -> impl Iterator<Item = AsteroidSize> {
        let budget = (self.0 % 20 + 2) * AsteroidSize::Large.cost();
        self.asteroid_sizes()
            .iter()
            .cycle()
            .scan(budget, move |budget, &size| {
                if *budget >= size.cost() {
                    *budget -= size.cost();
                    Some(size)
                } else if *budget > 0 {
                    *budget -= 1;
                    Some(AsteroidSize::Tiny)
                } else {
                    None
                }
            })
    }
    pub fn ufo_duration(&self) -> f32 {
        lerp(20.0, 10.0, self.0 as f32 / 40.0)
    }
    pub fn ufo_shoot_delay(&self) -> f32 {
        lerp(3.0, 1.5, self.0 as f32 / 60.0)
    }
    pub fn ufo_shoot_accuracy(&self) -> f32 {
        lerp(0.6, 0.9, self.0 as f32 / 60.0)
    }
    pub fn asteroid_distance_bounds(&self) -> std::ops::RangeInclusive<f32> {
        100.0..=200.0
    }
    pub fn asteroid_sizes(&self) -> &'static [AsteroidSize] {
        match self.0 {
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
        }
    }
    pub fn asteroid_speed_bounds(&self) -> std::ops::RangeInclusive<f32> {
        let min = lerp(10.0, 20.0, self.0 as f32 / 40.0);
        let max = lerp(20.0, 60.0, self.0 as f32 / 40.0);
        min..=max
    }
    pub fn asteroid_frag_count(&self) -> u32 {
        2 + self.0 % 20
    }
}

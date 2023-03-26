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

#[derive(Resource)]
pub struct GameState {
    pub level: u32,
    pub score: u32,
    pub next_ufo_score: u32,
}

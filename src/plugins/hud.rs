use crate::{AppState, components::*, resources::*};
use bevy::prelude::*;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_hud_system,
                update_hud_text_system.after(update_hud_system),
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}
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
    pub changed: bool,
}

fn update_hud_system(
    ships_query: Query<&Ship>,
    score: Res<Score>,
    level: Res<Level>,
    mut hud_query: Query<&mut HUD>,
    mut commands: Commands,
) {
    let Ok(ship) = ships_query.single() else {
        return;
    };
    let new_hud = HUD {
        level: level.number(),
        score: score.value(),
        lives: ship.lives,
        weapon: ship.weapon,
        weapon_rapid_level: ship.weapon_rapid_level,
        weapon_spread_level: ship.weapon_spread_level,
        weapon_beam_level: ship.weapon_beam_level,
        weapon_plasma_level: ship.weapon_plasma_level,
        changed: false,
    };
    if let Ok(mut hud) = hud_query.single_mut() {
        if *hud != new_hud {
            *hud = new_hud;
            hud.changed = true;
        }
    } else {
        commands.spawn(HUD {
            changed: true,
            ..new_hud
        });
    }
}

fn update_hud_text_system(
    mut commands: Commands,
    mut hud_query: Query<(Entity, &HUD)>,
    asset_server: Res<AssetServer>,
) {
    // FIXME: The HUD system originally user Changed<HUD> to update the Text.sections
    //        but for some reasons that caused the HUD to sometimes not render at all
    //        Creating a new text bundle for every update and using the changed property
    //        for HUD is a workaround that seems to work.
    let Ok((entity, hud)) = hud_query.single_mut() else {
        return;
    };
    if !hud.changed {
        return;
    }
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

    let font = asset_server.load("fonts/DejaVuSans.ttf");
    commands.entity(entity).despawn();
    commands.spawn((
        Text::new(hud_text),
        TextColor(Color::WHITE.with_alpha(0.7)),
        TextShadow {
            offset: Vec2::splat(1.0),
            color: Color::BLACK,
        },
        TextFont {
            font,
            font_size: 20.0,
            ..Default::default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(8.0),
            ..Default::default()
        },
        HUD {
            changed: false,
            ..*hud
        },
        LevelEntity,
    ));
}

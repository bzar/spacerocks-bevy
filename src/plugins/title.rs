use crate::AppState;
use bevy::prelude::*;

#[derive(Component)]
pub struct TitleEntity;

#[derive(Component)]
pub struct TitleText {
    from: Vec2,
    to: Vec2,
    at: f32,
    duration: f32,
    elapsed: f32,
}
#[derive(Component)]
pub struct TitleStart {
    at: f32,
    blink: f32,
    elapsed: f32,
}

fn init_title(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background = asset_server.load("img/title-background.png");
    commands
        .spawn(SpriteBundle {
            texture: background,
            ..default()
        })
        .insert(TitleEntity);

    let space = asset_server.load("img/title-space.png");
    commands
        .spawn(SpriteBundle {
            texture: space,
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(TitleEntity)
        .insert(TitleText {
            from: Vec2::new(-800.0, -50.0),
            to: Vec2::new(-150.0, 50.0),
            at: 1.0,
            duration: 0.5,
            elapsed: 0.0,
        });
    let rocks = asset_server.load("img/title-rocks.png");
    commands
        .spawn(SpriteBundle {
            texture: rocks,
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(TitleEntity)
        .insert(TitleText {
            from: Vec2::new(700.0, 50.0),
            to: Vec2::new(0.0, -50.0),
            at: 1.5,
            duration: 0.5,
            elapsed: 0.0,
        });
    let exclamation = asset_server.load("img/title-exclamation.png");
    commands
        .spawn(SpriteBundle {
            texture: exclamation,
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(TitleEntity)
        .insert(TitleText {
            from: Vec2::new(450.0, 370.0),
            to: Vec2::new(320.0, 80.0),
            at: 2.2,
            duration: 0.3,
            elapsed: 0.0,
        });
    let start = asset_server.load("img/title-start.png");
    commands
        .spawn(SpriteBundle {
            texture: start,
            transform: Transform::from_xyz(0.0, -200.0, 0.01),
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(TitleEntity)
        .insert(TitleStart {
            at: 2.8,
            blink: 0.2,
            elapsed: 0.0,
        });
}
fn title_input(keyboard_input: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(AppState::NewGame)
    }
}
fn title_text_system(
    mut title_text_query: Query<(&mut TitleText, &mut Transform, &mut Visibility)>,
    time: Res<Time>,
) {
    for (mut text, mut transform, mut visibility) in title_text_query.iter_mut() {
        text.elapsed += time.delta_seconds();
        *visibility = if text.elapsed >= text.at {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let t = (text.elapsed - text.at).clamp(0.0, text.duration) / text.duration;
        transform.translation = text.from.lerp(text.to, t).extend(0.01);
    }
}
fn title_start_system(
    mut title_start_query: Query<(&mut TitleStart, &mut Visibility)>,
    time: Res<Time>,
) {
    for (mut start, mut visibility) in title_start_query.iter_mut() {
        start.elapsed += time.delta_seconds();
        *visibility = if (start.elapsed - start.at)
            .max(0.0)
            .rem_euclid(start.blink * 2.0)
            >= start.blink
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
pub struct TitleScreenPlugin;
impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(init_title.in_schedule(OnEnter(AppState::Title)))
            .add_system(crate::despawn_tagged::<TitleEntity>.in_schedule(OnExit(AppState::Title)))
            .add_systems(
                (title_input, title_text_system, title_start_system)
                    .in_set(OnUpdate(AppState::Title)),
            );
    }
}

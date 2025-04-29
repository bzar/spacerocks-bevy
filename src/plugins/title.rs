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
    commands.spawn((
        Sprite::from_image(asset_server.load("img/title-background.png")),
        TitleEntity,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("img/title-space.png")),
        Visibility::Hidden,
        TitleEntity,
        TitleText {
            from: Vec2::new(-800.0, -50.0),
            to: Vec2::new(-150.0, 50.0),
            at: 1.0,
            duration: 0.5,
            elapsed: 0.0,
        },
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("img/title-rocks.png")),
        Visibility::Hidden,
        TitleEntity,
        TitleText {
            from: Vec2::new(700.0, 50.0),
            to: Vec2::new(0.0, -50.0),
            at: 1.5,
            duration: 0.5,
            elapsed: 0.0,
        },
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("img/title-exclamation.png")),
        Visibility::Hidden,
        TitleEntity,
        TitleText {
            from: Vec2::new(450.0, 370.0),
            to: Vec2::new(320.0, 80.0),
            at: 2.2,
            duration: 0.3,
            elapsed: 0.0,
        },
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("img/title-start.png")),
        Transform::from_xyz(0.0, -200.0, 0.01),
        Visibility::Hidden,
        TitleEntity,
        TitleStart {
            at: 2.8,
            blink: 0.2,
            elapsed: 0.0,
        },
    ));
}
fn title_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(AppState::NewGame)
    }
}
fn title_text_system(
    mut title_text_query: Query<(&mut TitleText, &mut Transform, &mut Visibility)>,
    time: Res<Time>,
) {
    for (mut text, mut transform, mut visibility) in title_text_query.iter_mut() {
        text.elapsed += time.delta_secs();
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
        start.elapsed += time.delta_secs();
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
        app.add_systems(OnEnter(AppState::Title), init_title)
            .add_systems(
                OnExit(AppState::Title),
                crate::despawn_tagged::<TitleEntity>,
            )
            .add_systems(
                Update,
                (title_input, title_text_system, title_start_system)
                    .run_if(in_state(AppState::Title)),
            );
    }
}

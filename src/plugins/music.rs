use crate::AppState;
use bevy::prelude::*;

pub struct MusicPlugin;
impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Music::default())
            .add_systems(Startup, init)
            .add_systems(OnEnter(AppState::NewGame), new_game_music)
            .add_systems(OnEnter(AppState::Title), title_music)
            .add_systems(OnEnter(AppState::HighScoreEntry), highscore_music);
    }
}

#[derive(Resource, Default)]
pub struct Music {
    title: Handle<AudioSource>,
    level: Handle<AudioSource>,
}

fn init(asset_server: Res<AssetServer>, mut music: ResMut<Music>) {
    music.title = asset_server.load::<AudioSource>("snd/music/title.ogg");
    music.level = asset_server.load::<AudioSource>("snd/music/01.ogg");
}

fn title_music(mut commands: Commands, music: Res<Music>, sinks: Query<Entity, With<AudioSink>>) {
    for sink in sinks.iter() {
        commands.entity(sink).despawn();
    }
    commands.spawn((
        AudioPlayer(music.title.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
    ));
}
fn new_game_music(
    mut commands: Commands,
    music: Res<Music>,
    sinks: Query<Entity, With<AudioSink>>,
) {
    for sink in sinks.iter() {
        commands.entity(sink).despawn();
    }
    commands.spawn((
        AudioPlayer(music.level.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
    ));
}
fn highscore_music(mut commands: Commands, sinks: Query<Entity, With<AudioSink>>) {
    for sink in sinks.iter() {
        commands.entity(sink).despawn();
    }
}

use crate::AppState;
use bevy::prelude::*;

pub struct MusicPlugin;
impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Music::default())
            .add_systems(Update, next_track_system)
            .add_systems(Startup, init)
            .add_systems(OnEnter(AppState::NewGame), new_game_music)
            .add_systems(OnEnter(AppState::Title), title_music)
            .add_systems(OnEnter(AppState::HighScoreEntry), highscore_music);
    }
}

#[derive(Resource, Default)]
pub struct Music {
    title: Handle<AudioSource>,
    level: Vec<Handle<AudioSource>>,
}

#[derive(Component)]
pub enum MusicPlayer {
    Title,
    Level(usize),
}

impl Music {
    fn get_source(&self, player: &MusicPlayer) -> Handle<AudioSource> {
        match player {
            MusicPlayer::Title => self.title.clone(),
            MusicPlayer::Level(track) => self.level[*track].clone(),
        }
    }
    fn next_track(&self, player: &MusicPlayer) -> MusicPlayer {
        match player {
            MusicPlayer::Title => MusicPlayer::Title,
            MusicPlayer::Level(track) => MusicPlayer::Level((track + 1) % self.level.len()),
        }
    }
}
fn init(asset_server: Res<AssetServer>, mut music: ResMut<Music>) {
    music.title = asset_server.load::<AudioSource>("snd/music/title.ogg");
    music.level = [
        "snd/music/01.ogg",
        "snd/music/02.ogg",
        "snd/music/03.ogg",
        "snd/music/04.ogg",
        "snd/music/05.ogg",
        "snd/music/06.ogg",
        "snd/music/07.ogg",
        "snd/music/08.ogg",
        "snd/music/09.ogg",
    ]
    .map(|path| asset_server.load::<AudioSource>(path))
    .into_iter()
    .collect();
}

fn title_music(mut commands: Commands, music: Res<Music>, sinks: Query<Entity, With<MusicPlayer>>) {
    for sink in sinks.iter() {
        commands.entity(sink).despawn();
    }
    commands.spawn((AudioPlayer(music.title.clone()), MusicPlayer::Title));
}
fn new_game_music(
    mut commands: Commands,
    music: Res<Music>,
    players: Query<Entity, With<MusicPlayer>>,
) {
    for player in players.iter() {
        commands.entity(player).despawn();
    }
    let player = MusicPlayer::Level(0);
    commands.spawn((AudioPlayer(music.get_source(&player)), player));
}
fn highscore_music(mut commands: Commands, sinks: Query<Entity, With<MusicPlayer>>) {
    for sink in sinks.iter() {
        commands.entity(sink).despawn();
    }
}

fn next_track_system(
    mut commands: Commands,
    players: Query<(Entity, &MusicPlayer, &AudioSink)>,
    music: Res<Music>,
) {
    for (entity, player, sink) in players.iter() {
        if sink.empty() {
            commands.entity(entity).despawn();
            let new_player = music.next_track(player);
            let source = music.get_source(&new_player);
            commands.spawn((AudioPlayer(source), new_player));
        }
    }
}

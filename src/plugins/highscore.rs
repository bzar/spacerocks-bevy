use crate::components::Fading;
use crate::constants::*;
use crate::input::InputState;
use crate::resources::Score;
use crate::AppState;
use bevy::prelude::*;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Component)]
struct HighScoreEntity;

#[derive(Component)]
struct HighScoreEntryLetter {
    index: i32,
    blinking: bool,
}

pub struct HighScoreEntry {
    name: String,
    score: u32,
}

#[derive(Resource)]
pub struct HighScore {
    pub entries: Vec<HighScoreEntry>,
}

pub struct HighScorePlugin;
impl Plugin for HighScorePlugin {
    fn build(&self, app: &mut App) {
        let high_score = HighScore::load().unwrap_or(HighScore {
            entries: Vec::new(),
        });
        app.insert_resource(high_score)
            .add_system(init_highscore.in_schedule(OnEnter(AppState::HighScore)))
            .add_system(
                crate::despawn_tagged::<HighScoreEntity>.in_schedule(OnExit(AppState::HighScore)),
            )
            .add_systems((highscore_input,).in_set(OnUpdate(AppState::HighScore)))
            .add_system(init_highscore_entry.in_schedule(OnEnter(AppState::HighScoreEntry)))
            .add_system(
                crate::despawn_tagged::<HighScoreEntity>
                    .in_schedule(OnExit(AppState::HighScoreEntry)),
            )
            .add_systems(
                (highscore_entry_input, highscore_entry_letter_blink)
                    .in_set(OnUpdate(AppState::HighScoreEntry)),
            );
    }
}

fn init_highscore(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    high_score: Res<HighScore>,
) {
    let texture = asset_server.load("img/highscores.png");
    commands
        .spawn(SpriteBundle {
            texture,
            ..default()
        })
        .insert(HighScoreEntity);
    let entries = high_score.entries.len() as i32;
    let rows_per_column = 5;
    let columns = entries / (rows_per_column + 1) + 1;
    let column_padding = if entries % columns == 0 { 0 } else { 1 };
    let column_size = entries / columns + column_padding;

    let font = asset_server.load("fonts/DejaVuSans.ttf");
    for (i, entry) in high_score.entries.iter().enumerate() {
        let column = (i as i32 / column_size) as f32;
        let row = (i as i32 % column_size) as f32;
        let position = Vec2::new(
            (column + 0.5) * GAME_WIDTH as f32 / columns as f32 - GAME_WIDTH as f32 / 2.0,
            -(row + 0.5) * 40.0,
        );
        commands
            .spawn(HighScoreText::new(
                position,
                i as u32 + 1,
                &entry.name,
                entry.score,
                font.clone(),
            ))
            .insert(HighScoreEntity);
    }
}

fn highscore_input(input: Res<InputState>, mut next_state: ResMut<NextState<AppState>>) {
    if input.ok {
        next_state.set(AppState::Title)
    }
}

fn init_highscore_entry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    high_score: Res<HighScore>,
    score: Res<Score>,
) {
    let texture = asset_server.load("img/gameover.png");
    commands
        .spawn(SpriteBundle {
            texture,
            ..default()
        })
        .insert(HighScoreEntity);

    let is_high_score = high_score.entries.len() < MAX_HIGH_SCORE_ENTRIES
        || high_score.entries.iter().any(|hs| hs.score < score.value());

    let font = asset_server.load("fonts/DejaVuSans.ttf");
    if is_high_score {
        info!("New high score!");
        for i in 0..NUM_HIGH_SCORE_ENTRY_LETTERS {
            let x = (i as i32 * 40 - (NUM_HIGH_SCORE_ENTRY_LETTERS + 1) * 20) as f32;
            commands
                .spawn(Text2dBundle {
                    text: Text::from_section(
                        "A".to_string(),
                        TextStyle {
                            font: font.clone(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ),
                    transform: Transform::from_xyz(x, -70.0, 0.01),
                    ..default()
                })
                .insert(HighScoreEntryLetter {
                    index: i,
                    blinking: i == 0,
                })
                .insert(HighScoreEntity);
        }
    }
}

fn highscore_entry_input(
    mut letters: Query<(&mut HighScoreEntryLetter, &mut Text)>,
    input: Res<InputState>,
    mut selected: Local<i32>,
    mut next_state: ResMut<NextState<AppState>>,
    mut high_score: ResMut<HighScore>,
    score: Res<Score>,
) {
    if input.ok {
        if letters.is_empty() {
            next_state.set(AppState::HighScore);
        } else {
            *selected += 1;
            if *selected == NUM_HIGH_SCORE_ENTRY_LETTERS {
                // FIXME: This is horrible, but it works
                let mut indexed_letters: Vec<_> = letters
                    .iter()
                    .map(|(letter, text)| (letter.index, &text.sections[0].value))
                    .collect();
                indexed_letters.sort_unstable_by_key(|(index, _)| *index);
                let name: String = indexed_letters
                    .iter()
                    .map(|&(_, letter)| letter.clone())
                    .collect();
                high_score.entries.push(HighScoreEntry {
                    name,
                    score: score.value(),
                });
                high_score
                    .entries
                    .sort_by_key(|entry| -(entry.score as i64));
                high_score
                    .entries
                    .truncate(NUM_HIGH_SCORE_ENTRY_LETTERS as usize);
                high_score.save().expect("Could not save high score!");
                *selected = 0;
                next_state.set(AppState::HighScore);
            }
        }
    }
    const CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    for (mut letter, mut text) in letters.iter_mut() {
        let selected = letter.index == *selected;
        letter.blinking = selected;
        if selected {
            if input.up {
                let ch = text.sections[0].value.chars().next().unwrap();
                let new_ch = CHARS
                    .chars()
                    .cycle()
                    .skip_while(|c| *c != ch)
                    .skip(1)
                    .next()
                    .unwrap();
                text.sections[0].value = new_ch.to_string();
            }
            if input.down {
                let ch = text.sections[0].value.chars().next().unwrap();
                let new_ch = CHARS
                    .chars()
                    .rev()
                    .cycle()
                    .skip_while(|c| *c != ch)
                    .skip(1)
                    .next()
                    .unwrap();
                text.sections[0].value = new_ch.to_string();
            }
        }
    }
}

fn highscore_entry_letter_blink(
    mut letters: Query<(&HighScoreEntryLetter, &mut Visibility)>,
    time: Res<Time>,
) {
    for (letter, mut visibility) in letters.iter_mut() {
        *visibility = if letter.blinking && time.elapsed_seconds_wrapped().rem_euclid(0.4) < 0.2 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        }
    }
}

#[derive(Bundle)]
struct HighScoreText {
    text: Text2dBundle,
    fading: Fading,
}
impl HighScoreText {
    fn new(position: Vec2, rank: u32, name: &str, score: u32, font: Handle<Font>) -> Self {
        HighScoreText {
            text: Text2dBundle {
                text: Text::from_section(
                    format!("{rank}. {name} - {score}"),
                    TextStyle {
                        font,
                        font_size: 32.0,
                        color: Color::WHITE,
                    },
                ),
                transform: Transform::from_translation(position.extend(0.1)),
                ..default()
            },
            fading: Fading {
                from: 0.0,
                to: 1.0,
                duration: rank as f32,
                elapsed: 0.0,
            },
        }
    }
}

impl HighScore {
    fn crypt(content: &[u8]) -> Vec<u8> {
        let key = "Space Rocks!".as_bytes().into_iter().cycle();
        content.iter().zip(key).map(|(a, b)| a ^ b).collect()
    }
    fn save(&self) -> std::io::Result<()> {
        let content: String = self
            .entries
            .iter()
            .map(|e| format!("{}:{}\n", e.name, e.score))
            .collect();
        let encoded = HighScore::crypt(&content.as_bytes());
        let mut file = File::create("highscore.enc")?;
        file.write_all(&encoded)?;
        Ok(())
    }
    fn load() -> std::io::Result<Self> {
        let mut file = File::open("highscore.enc")?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        let decoded = HighScore::crypt(&content);
        let entries: Vec<_> = std::str::from_utf8(&decoded)
            .expect("Invalid high score file!")
            .split(|ch| ch == '\n')
            .filter_map(|e| e.split_once(':'))
            .map(|(name, score_str)| HighScoreEntry {
                name: name.to_string(),
                score: score_str.parse().expect("Invalid high score file!"),
            })
            .collect();
        Ok(HighScore { entries })
    }
}

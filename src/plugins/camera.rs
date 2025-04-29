use bevy::{prelude::*, render::camera::Viewport};

use crate::constants::*;

fn add_camera(mut commands: Commands, window_query: Query<&Window>) {
    let window = window_query.single().unwrap();
    commands.spawn((
        Camera2d,
        Camera {
            viewport: Some(window_to_viewport(window, GAME_WIDTH, GAME_HEIGHT)),
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            near: -1.0,
            scaling_mode: bevy::render::camera::ScalingMode::AutoMin {
                min_width: GAME_WIDTH as f32,
                min_height: GAME_HEIGHT as f32,
            },
            area: Rect::from_center_size(Vec2::ZERO, Vec2::new(800.0, 480.0)),
            ..OrthographicProjection::default_2d()
        }),
    ));
}
fn window_to_viewport(window: &Window, width: u32, height: u32) -> Viewport {
    let physical_size = UVec2::new(
        window
            .physical_width()
            .min(window.physical_height() * width / height),
        window
            .physical_height()
            .min(window.physical_width() * height / width),
    );
    let physical_position = UVec2::new(
        (window.physical_width().max(physical_size.x) - physical_size.x) / 2,
        (window.physical_height().max(physical_size.y) - physical_size.y) / 2,
    );
    Viewport {
        physical_position,
        physical_size,
        ..default()
    }
}
fn viewport_system(mut camera_query: Query<&mut Camera>, window_query: Query<&Window>) {
    let mut camera = camera_query.single_mut().unwrap();
    let window = window_query.single().unwrap();
    camera.viewport = Some(window_to_viewport(window, GAME_WIDTH, GAME_HEIGHT));
}
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_camera)
            .add_systems(Update, viewport_system);
    }
}

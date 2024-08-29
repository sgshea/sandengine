//! The screen state for the main game loop.

use bevy::{input::common_conditions::input_just_pressed, prelude::*, render::view::RenderLayers};

use crate::{spawn_worlds, WorldSizes};

use super::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), enter_playing);

    app.add_systems(
        Update,
        return_to_title_screen
            .run_if(in_state(Screen::Playing).and_then(input_just_pressed(KeyCode::Escape))),
    );

    app.add_systems(Startup, spawn_ui_camera);
    app.add_systems(OnEnter(Screen::Title), spawn_ui_camera);
}

fn enter_playing(mut commands: Commands, world_size: Res<State<WorldSizes>>) {
    spawn_worlds(&mut commands, world_size)
}

fn return_to_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn spawn_ui_camera(mut commands: Commands, camera_query: Query<Entity, With<IsDefaultUiCamera>>) {
    // Make sure camera does not already exist
    match camera_query.get_single() {
        Ok(_) => {}
        Err(_) => {
            commands.spawn((
                Name::new("Camera"),
                Camera2dBundle::default(),
                IsDefaultUiCamera,
                // 5 is render layer for main menu ui
                RenderLayers::from_layers(&[5]),
            ));
        }
    };
}

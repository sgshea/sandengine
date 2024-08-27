//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on WASM.

use bevy::prelude::*;

use super::Screen;
use crate::{states::AppSet, ui::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), enter_loading);

    app.register_type::<LoadingTimer>();
    app.add_systems(OnEnter(Screen::Loading), insert_loading_timer);
    app.add_systems(OnExit(Screen::Loading), remove_loading_timer);
    app.add_systems(
        Update,
        (
            tick_loading_timer.in_set(AppSet::TickTimers),
            check_loading_timer.in_set(AppSet::Update),
        )
            .run_if(in_state(Screen::Loading)),
    );
}

const LOADING_DURATION_SECS: f32 = 0.8;

fn enter_loading(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Loading))
        .with_children(|children| {
            children.label("Loading...");
        });
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct LoadingTimer(Timer);

impl Default for LoadingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(LOADING_DURATION_SECS, TimerMode::Once))
    }
}

fn insert_loading_timer(mut commands: Commands) {
    commands.init_resource::<LoadingTimer>();
}

fn remove_loading_timer(mut commands: Commands) {
    commands.remove_resource::<LoadingTimer>();
}

fn tick_loading_timer(time: Res<Time>, mut timer: ResMut<LoadingTimer>) {
    timer.0.tick(time.delta());
}

fn check_loading_timer(timer: ResMut<LoadingTimer>, mut next_screen: ResMut<NextState<Screen>>) {
    if timer.0.just_finished() {
        next_screen.set(Screen::Playing);
    }
}

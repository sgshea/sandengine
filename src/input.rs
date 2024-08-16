use bevy::prelude::*;

use crate::states::DebugState;

pub(super) fn plugin(app: &mut App) {
        app.add_systems(Update, handle_keyboard_input);
}

fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    debug_state: Res<State<DebugState>>,
    mut next_debug_state: ResMut<NextState<DebugState>>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        match debug_state.get() {
            DebugState::None => next_debug_state.set(DebugState::ShowAll),
            DebugState::ShowAll => next_debug_state.set(DebugState::None),
        }
    }
}
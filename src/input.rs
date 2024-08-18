use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;

use crate::states::DebugState;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(InteractionInformation::default());
    app.add_systems(Update, (get_position, handle_keyboard_input));
}

#[derive(Resource, Default)]
pub struct InteractionInformation {
    pub mouse_position: Vec2,
    pub hovering_ui: bool,
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

// Gets the position in the world that the mouse is hovering over
fn get_position(
    mut int: ResMut<InteractionInformation>,
    mut egui_ctx: EguiContexts,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let cursor_screen_position = primary_window.single().cursor_position();

    int.hovering_ui = egui_ctx.ctx_mut().wants_pointer_input();

    if cursor_screen_position.is_none() || int.hovering_ui {
        return
    }
    let (cam, trans) = camera.single();

    int.mouse_position = cam.viewport_to_world_2d(trans, cursor_screen_position.unwrap()).unwrap();
}
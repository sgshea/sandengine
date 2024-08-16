use bevy_rapier2d::render::{DebugRenderContext, RapierDebugRenderPlugin};
use bevy::prelude::*;

use crate::states::DebugState;

pub(super) fn plugin(app: &mut App) {
        app
            .add_plugins(RapierDebugRenderPlugin::default())
            .init_resource::<PixelSimulationDebugUi>()
            .add_systems(Update, handle_keyboard_input);
}


#[derive(Resource, Default)]
pub struct PixelSimulationDebugUi {
    pub show: bool,
}

fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    debug_state: Res<State<DebugState>>,
    mut rapier_debug_ctx: ResMut<DebugRenderContext>,
    mut pxl_dbg_ui: ResMut<PixelSimulationDebugUi>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        rapier_debug_ctx.enabled = !rapier_debug_ctx.enabled;
    }
    if debug_state.is_changed() {
        match debug_state.get() {
            DebugState::None => {
                pxl_dbg_ui.show = false;
            },
            DebugState::ShowAll => {
                pxl_dbg_ui.show = true;
            },
        }
    }
}
use bevy_rapier2d::render::RapierDebugRenderPlugin;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
        app
            .add_plugins(RapierDebugRenderPlugin::default());
}
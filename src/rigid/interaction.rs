// Interaction with rigid bodies

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use strum::{EnumIter, IntoEnumIterator, VariantNames};

use crate::input::InteractionInformation;

use super::rigidbodies::{add_non_dynamic_rigidbody, RigidBodyImageHandle};

#[derive(Resource, Default)]
pub struct RigidInteraction {
    // Type of rigid body to be placed on click
    pub place_rigid_type: PlaceableRigidBodies,
}

#[derive(Debug, Default, EnumIter, VariantNames, PartialEq, Eq, Clone, Copy)]
pub enum PlaceableRigidBodies {
    None,
    #[default]
    Ball,
    Box,
}

pub(super) fn plugin(app: &mut App) {
        app.init_resource::<RigidInteraction>();
        app.add_systems(Update, rigid_interaction_config);
        app.add_systems(Update, handle_input);
}

fn rigid_interaction_config(
    mut ctx: EguiContexts,
    mut rgd: ResMut<RigidInteraction>,
) {
    egui::Window::new("Rigid Body Simulation").show(ctx.ctx_mut(),
        | ui | {
            ui.set_min_width(100.);
            ui.label("Controls:");
            ui.label("Right click to place the selected type");
            ui.separator();
            for (rigid_type, name) in PlaceableRigidBodies::iter().zip(PlaceableRigidBodies::VARIANTS.iter()) {
                ui.radio_value(&mut rgd.place_rigid_type, rigid_type, *name);

            }
            ui.separator();
            ui.label("Press F1 to toggle debug window");
        }
    );
}

fn handle_input(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    rgd: Res<RigidInteraction>,
    int: Res<InteractionInformation>,
) {
    if !int.hovering_ui && mouse_button_input.just_released(MouseButton::Right) {
        add_non_dynamic_rigidbody(commands, meshes, materials, int.mouse_position.as_ivec2(), rgd.place_rigid_type);
    }
}
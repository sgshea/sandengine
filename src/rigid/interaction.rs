// Interaction with rigid bodies

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use strum::{EnumIter, IntoEnumIterator, VariantNames};

use crate::{input::InteractionInformation, screen::Screen};

use super::{
    dynamic_entity::{add_dpe, RigidBodyImageHandle},
    rigidbodies::add_non_dynamic_rigidbody,
};

#[derive(Resource, Default)]
pub struct RigidInteraction {
    // Type of rigid body to be placed on click
    pub place_rigid_type: PlaceableRigidBodies,

    // Type of dynamic physics entity to be placed on click
    pub place_dynamic_entity_type: PlaceableDynamicEntities,
}

#[derive(Debug, Default, EnumIter, VariantNames, PartialEq, Eq, Clone, Copy)]
pub enum PlaceableRigidBodies {
    None,
    #[default]
    Ball,
    Box,
}

#[derive(Debug, Default, EnumIter, VariantNames, PartialEq, Eq, Clone, Copy)]
pub enum PlaceableDynamicEntities {
    None,
    #[default]
    Box,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RigidInteraction>();
    app.add_systems(
        Update,
        (rigid_interaction_config, handle_input).run_if(in_state(Screen::Playing)),
    );
}

fn rigid_interaction_config(mut ctx: EguiContexts, mut rgd: ResMut<RigidInteraction>) {
    egui::Window::new("Rigid Body Simulation").show(ctx.ctx_mut(), |ui| {
        ui.group(|ui| {
            ui.label("Right click:\nPlace a Dynamic Physics Body");
            for (dpe_type, name) in
                PlaceableDynamicEntities::iter().zip(PlaceableDynamicEntities::VARIANTS.iter())
            {
                ui.radio_value(&mut rgd.place_dynamic_entity_type, dpe_type, *name);
            }
        });
        ui.group(|ui| {
            ui.label("Left Control + Right click:\nPlace non-interacting physics body.");
            for (rigid_type, name) in
                PlaceableRigidBodies::iter().zip(PlaceableRigidBodies::VARIANTS.iter())
            {
                ui.radio_value(&mut rgd.place_rigid_type, rigid_type, *name);
            }
        });
    });
}

fn handle_input(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_buttons: Res<ButtonInput<KeyCode>>,
    rgd: Res<RigidInteraction>,
    int: Res<InteractionInformation>,

    images: Res<Assets<Image>>,
    rigidbody_image: Res<RigidBodyImageHandle>,
) {
    if !int.hovering_ui && mouse_button_input.just_released(MouseButton::Right) {
        // Place DPE with control held
        if keyboard_buttons.pressed(KeyCode::ControlLeft) {
            if keyboard_buttons.pressed(KeyCode::ShiftLeft) {
                // Add 10
                for _ in 0..10 {
                    add_non_dynamic_rigidbody(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        int.mouse_position.as_ivec2(),
                        rgd.place_rigid_type,
                    );
                }
            } else {
                add_non_dynamic_rigidbody(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    int.mouse_position.as_ivec2(),
                    rgd.place_rigid_type,
                );
            }
        } else {
            if keyboard_buttons.pressed(KeyCode::ShiftLeft) {
                // Add 10
                for _ in 0..10 {
                    add_dpe(&mut commands, &images, int.mouse_position, &rigidbody_image);
                }
            } else {
                add_dpe(&mut commands, &images, int.mouse_position, &rigidbody_image);
            }
        }
    }
}

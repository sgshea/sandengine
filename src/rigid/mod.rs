//! Rigid body module which implements a plugin that handles rigid bodies created and managed by the Rapier physics engine.
//! Handles interactions between the rigid bodies and pixel simulation world

mod character_control_tnua;
mod collider_generation;
pub mod dynamic_entity;
mod interaction;
mod rigidbodies;

use std::f32::consts::FRAC_PI_4;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_tnua::{
    builtins::{TnuaBuiltinJump, TnuaBuiltinWalk},
    control_helpers::{TnuaCrouchEnforcer, TnuaCrouchEnforcerPlugin, TnuaSimpleAirActionsCounter},
    controller::{TnuaControllerBundle, TnuaControllerPlugin},
    math::Vector3,
    TnuaGhostSensor, TnuaToggle, TnuaUserControlsSystemSet,
};
use bevy_tnua_rapier2d::{TnuaRapier2dIOBundle, TnuaRapier2dPlugin, TnuaRapier2dSensorShape};
use character_control_tnua::{apply_platformer_controls, CharacterMotionConfigForPlatformer};
use collider_generation::chunk_collider_generation;
use dynamic_entity::{
    fill_pixel_component, load_rigidbody_image, unfill_pixel_component, RigidBodyImageHandle,
};

use crate::{pixel::update_pixel_simulation, screen::Screen, SpawnWorlds};

pub struct SandEngineRigidPlugin;

impl Plugin for SandEngineRigidPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RigidStorage {
            colliders: Vec::new(),
        })
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.))
        .add_plugins((
            TnuaRapier2dPlugin::new(FixedUpdate),
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaCrouchEnforcerPlugin::new(FixedUpdate),
            interaction::plugin,
        ))
        .add_systems(Startup, |mut cfg: ResMut<RapierConfiguration>| {
            cfg.gravity = Vec2::Y * -9.81;
        })
        .insert_resource(RigidBodyImageHandle { handle: None })
        .add_systems(Startup, load_rigidbody_image)
        .add_systems(
            FixedUpdate.intern(),
            apply_platformer_controls
                .in_set(TnuaUserControlsSystemSet)
                .run_if(in_state(Screen::Playing)),
        )
        .add_systems(
            FixedUpdate,
            (
                fill_pixel_component.before(update_pixel_simulation),
                unfill_pixel_component.after(update_pixel_simulation),
                chunk_collider_generation,
            )
                .chain()
                .run_if(in_state(Screen::Playing)),
        );
    }
}

// RigidStorage is a resource that stores a vector for each chunk that contains the entities of the colliders in that chunk
#[derive(Resource)]
pub struct RigidStorage {
    // Static colliders generated from the pixel simulation
    pub colliders: Vec<Option<Vec<Entity>>>,
}

pub fn spawn_rigid_world(
    In(_config): In<SpawnWorlds>,
    mut commands: Commands,
    mut rigid_storage: ResMut<RigidStorage>,
) {
    setup_physics_environment(&mut commands);
    setup_player(&mut commands);

    // Reset rigid storage
    rigid_storage.colliders.clear();
}

// Setting simple stage
fn setup_physics_environment(commands: &mut Commands) {
    let mut cmd = commands.spawn(Name::new("Floor"));
    cmd.insert(Collider::halfspace(Vec2::Y).unwrap());
    // move the floor to the bottom of the screen
    cmd.insert(Transform::from_xyz(0.0, 0.0, 0.0));
    cmd.insert(StateScoped(Screen::Playing));
}

fn setup_player(commands: &mut Commands) {
    let mut cmd = commands.spawn_empty();
    cmd.insert(TransformBundle::from_transform(Transform::from_xyz(
        30.0, 10.0, 0.0,
    )));
    cmd.insert(VisibilityBundle::default());

    cmd.insert(RigidBody::Dynamic);
    cmd.insert(Collider::capsule_y(3.0, 1.0));
    // For Rapier, an "IO" bundle needs to be added so that Tnua will have all the components
    // it needs to interact with Rapier.
    cmd.insert(TnuaRapier2dIOBundle::default());

    cmd.insert(TnuaControllerBundle::default());

    cmd.insert(CharacterMotionConfigForPlatformer {
        speed: 80.0,
        walk: TnuaBuiltinWalk {
            float_height: 5.0,
            max_slope: FRAC_PI_4,
            ..Default::default()
        },
        actions_in_air: 2,
        jump: TnuaBuiltinJump {
            height: 25.0,
            ..Default::default()
        },
        dash_distance: 30.0,
        dash: Default::default(),
    });

    cmd.insert(TnuaToggle::default());
    cmd.insert(LockedAxes::ROTATION_LOCKED);

    // `TnuaCrouchEnforcer` can be used to prevent the character from standing up when obstructed.
    cmd.insert(TnuaCrouchEnforcer::new(0.5 * Vector3::Y, |cmd| {
        // It needs a sensor shape because it needs to do a shapecast upwards. Without a sensor shape
        // it'd do a raycast.
        cmd.insert(TnuaRapier2dSensorShape(Collider::cuboid(0.5, 0.0)));
    }));

    cmd.insert(TnuaGhostSensor::default());
    cmd.insert(TnuaSimpleAirActionsCounter::default());

    // By default Tnua uses a raycast, but this could be a problem if the character stands
    // just past the edge while part of its body is above the platform. To solve this, we
    // need to cast a shape - which is physics-engine specific. We set the shape using a
    // component.
    cmd.insert(TnuaRapier2dSensorShape(Collider::ball(0.70)));

    cmd.insert(StateScoped(Screen::Playing));
}

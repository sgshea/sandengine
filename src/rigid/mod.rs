mod rigidbodies;
mod character_control_tnua;

use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy::ecs::schedule::ScheduleLabel;
use bevy_rapier2d::prelude::*;
use bevy_tnua::{builtins::{TnuaBuiltinCrouch, TnuaBuiltinJump, TnuaBuiltinWalk}, control_helpers::{TnuaCrouchEnforcer, TnuaCrouchEnforcerPlugin, TnuaSimpleAirActionsCounter}, controller::{TnuaControllerBundle, TnuaControllerPlugin}, math::Vector3, TnuaGhostSensor, TnuaToggle, TnuaUserControlsSystemSet};
use bevy_tnua_rapier2d::{TnuaRapier2dIOBundle, TnuaRapier2dPlugin, TnuaRapier2dSensorShape};
use character_control_tnua::{apply_platformer_controls, CharacterMotionConfigForPlatformer};
use rigidbodies::generate_colliders;

use super::CHUNKS;

pub struct SandEngineRigidPlugin;

impl Plugin for SandEngineRigidPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RigidStorage::default())
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.).in_fixed_schedule())
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(FixedFirst, generate_colliders)
            .add_plugins((
                TnuaRapier2dPlugin::new(FixedUpdate),
                TnuaControllerPlugin::new(FixedUpdate),
                TnuaCrouchEnforcerPlugin::new(FixedUpdate),
            ))
            .add_systems(Startup, setup_player)
            .add_systems(Startup, |mut cfg: ResMut<RapierConfiguration>| {
                cfg.gravity = Vec2::Y * -9.81;
            })
            .add_systems(
                FixedUpdate.intern(),
                apply_platformer_controls.in_set(TnuaUserControlsSystemSet),
            );
    }
}

// RigidStorage is a resource that stores a vector for each chunk that contains the entities of the colliders in that chunk
#[derive(Resource)]
pub struct RigidStorage {
    pub colliders: Vec<Option<Vec<Entity>>>,
}

impl Default for RigidStorage {
    fn default() -> Self {
        Self {
            colliders: vec![None; CHUNKS.0 as usize * CHUNKS.1 as usize]
        }
    }
}

fn setup_player(mut commands: Commands) {

    let mut cmd = commands.spawn(Name::new("Floor"));
    cmd.insert(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(128.0, 0.5)),
            color: Color::GRAY,
            ..Default::default()
        },
        ..Default::default()
    });
    cmd.insert(Collider::halfspace(Vec2::Y).unwrap());

    for (name, [width, height], transform) in [
        (
            "Moderate Slope",
            [30.0, 0.1],
            Transform::from_xyz(17.0, 7.0, 0.0).with_rotation(Quat::from_rotation_z(0.6)),
        ),
        (
            "Steep Slope",
            [20.0, 0.1],
            Transform::from_xyz(24.0, 14.0, 0.0).with_rotation(Quat::from_rotation_z(1.0)),
        ),
        (
            "Box to Step on",
            [6.0, 2.0],
            Transform::from_xyz(-14.0, 1.0, 0.0),
        ),
        (
            "Floating Box",
            [8.0, 2.0],
            Transform::from_xyz(-20.0, 4.0, 0.0),
        ),
    ] {
        let mut cmd = commands.spawn(Name::new(name));
        cmd.insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(width, height)),
                color: Color::GRAY,
                ..Default::default()
            },
            transform,
            ..Default::default()
        });
        cmd.insert(Collider::cuboid(0.5 * width, 0.5 * height));
    }

    let mut cmd = commands.spawn_empty();
    cmd.insert(TransformBundle::from_transform(Transform::from_xyz(
        0.0, 10.0, 0.0,
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
        crouch: TnuaBuiltinCrouch {
            float_offset: -0.9,
            ..Default::default()
        },
        dash_distance: 30.0,
        dash: Default::default(),
        one_way_platforms_min_proximity: 1.0,
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
}
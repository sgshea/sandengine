use bevy::{prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}};
use bevy_rapier2d::prelude::*;

use crate::screen::Screen;

use super::interaction::PlaceableRigidBodies;

// Add a simple ball or box rigidbody to the world
pub fn add_non_dynamic_rigidbody(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: IVec2,
    rigid_type: PlaceableRigidBodies,
) {

    commands.spawn((
        match rigid_type {
            PlaceableRigidBodies::Ball => Collider::ball(3.0),
            PlaceableRigidBodies::Box => Collider::cuboid(5.0, 5.0),
            _ => return,
        },
        match rigid_type {
            PlaceableRigidBodies::Ball => {
                let mesh = Mesh2dHandle(meshes.add(Circle { radius: 3.0 }));
                MaterialMesh2dBundle {
                    mesh,
                    material: materials.add(Color::hsl((position.x * position.y) as f32, 0.95, 0.7)),
                    transform: Transform::from_translation(position.extend(0).as_vec3()),
                    ..Default::default()
                }
            },
            PlaceableRigidBodies::Box => {
                let mesh = Mesh2dHandle(meshes.add(Rectangle::new(10.0, 10.0)));
                MaterialMesh2dBundle {
                    mesh,
                    material: materials.add(Color::hsl((position.x * -position.y) as f32, 0.95, 0.7)),
                    transform: Transform::from_translation(position.extend(0).as_vec3()),
                    ..Default::default()
                }
            },
            _ => return,
        },
        RigidBody::Dynamic,
        ColliderMassProperties::default(),
        Restitution::coefficient(0.7),
        StateScoped(Screen::Playing),
    ));
}
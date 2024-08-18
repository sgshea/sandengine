use bevy::{prelude::*, sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle}};
use bevy_rapier2d::prelude::*;

use super::{collider_generation::create_collider, interaction::PlaceableRigidBodies};

#[derive(Resource)]
pub struct RigidBodyImageHandle {
    pub handle: Option<Handle<Image>>,
}

pub fn load_rigidbody_image(
    server: Res<AssetServer>,
    mut rigidbody_image: ResMut<RigidBodyImageHandle>,
) {
    let image_handle = server.load("images/box.png");
    rigidbody_image.handle = Some(image_handle);
}

// Add a single rigidbody to the world
pub fn add_rigidbody(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    rigidbody_image: Res<RigidBodyImageHandle>,
    position: IVec2,
) {
    let image_handle = rigidbody_image.handle.clone().unwrap();
    let image = images.get(&image_handle).unwrap();

    let values = image_valuemap(image);
    let collider = create_collider(&values, image.width(), image.height()).unwrap();
    
    // Create entity
    commands.spawn((
        // Collider constructed from image
        collider,
        // This is a rigidbody
        RigidBody::Dynamic,
        // Soft Continuous Collision Detection (CCD) to prevent tunneling
        // Soft CCD is cheaper than CCD using prediction
        SoftCcd {
            prediction: 15.0,
        },
        ColliderMassProperties::default(),
        Restitution::coefficient(0.7),
        // Giving some contact skin helps prevent tunnelling, jittering, and issues of rigidbodies going inside each other
        ContactSkin(0.5),
        // Image that the rigidbody is based on
        SpriteBundle {
            texture: image_handle.clone(),
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            transform: Transform::from_translation(position.extend(0).as_vec3()),
            ..Default::default()
        },
    ));
}

// Add a simple ball or box rigidbody to the world
pub fn add_non_dynamic_rigidbody(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    position: IVec2,
    rigid_type: PlaceableRigidBodies,
) {

    commands.spawn((
        match rigid_type {
            PlaceableRigidBodies::None => return,
            PlaceableRigidBodies::Ball => Collider::ball(3.0),
            PlaceableRigidBodies::Box => Collider::cuboid(5.0, 5.0),
        },
        match rigid_type {
            PlaceableRigidBodies::None => return,
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
        },
        RigidBody::Dynamic,
        ColliderMassProperties::default(),
        Restitution::coefficient(0.7),
        ContactSkin(0.5),
    ));
}

// Gets values to be used in the contour builder from the image based on the image's pixel values
fn image_valuemap(image: &Image) -> Vec<f64> {
    let mut values: Vec<f64> = Vec::new();
    for p in image.data.chunks_exact(4) {
        // If the pixel is not transparent, add it to the values
        if p[3] > 0 {
            values.push(1.0);
        } else {
            values.push(0.0);
        }
    }

    values
}
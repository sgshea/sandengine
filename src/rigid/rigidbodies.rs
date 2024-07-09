use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::*;


use super::collider_generation::create_collider;

// DynamicPhysicsEntity (DPE) is created each frame for the RigidBodyImageHandle, to faciliate interaction with the pixel engine
#[derive(Component)]
pub struct DynamicPhysicsEntity {
    pub width: u32,
    pub height: u32,

    pub position: Vec2,

    pub image: Option<Handle<Image>>,

    // Cells occupied by the entity in world space
    pub cells: Vec<(u32, u32)>,
}

#[derive(Resource)]
pub struct RigidBodyImageHandle {
    pub handle: Option<Handle<Image>>,
}

pub fn load_rigidbody_image(
    server: Res<AssetServer>,
    mut rigidbody_image: ResMut<RigidBodyImageHandle>,
) {
    let image_handle = server.load("box.png");
    rigidbody_image.handle = Some(image_handle);
}

// Add a single rigidbody to the world
pub fn add_rigidbody(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    rigidbody_image: Res<RigidBodyImageHandle>,
    position: Vec2,
) {
    let image_handle = rigidbody_image.handle.clone().unwrap();
    let image = images.get(&image_handle).unwrap();

    let values = image_valuemap(&image);
    let collider = create_collider(&values, image.width(), image.height()).unwrap();
    // let collider = Collider::cuboid(8., 8.);
    
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
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
            ..Default::default()
        },
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
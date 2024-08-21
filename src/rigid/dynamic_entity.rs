/// Type of rigid bodies that interact with the sand simulation

use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties, Restitution, RigidBody, Velocity};

use crate::pixel::cell::{Cell, CellType};

use super::collider_generation::create_convex_collider_from_values;

/// Bundle which includes physics properties along with the PixelComponent
#[derive(Bundle)]
pub struct DynamicPhysicsEntity {
    // Collider and Rigidbody of Rapier
    pub collider: Collider,
    pub rigidbody: RigidBody,

    // Modifiable properties of the rigidbody
    // TODO: custom generation of these properties based on the pixel data
    pub mass: ColliderMassProperties,
    pub restitution: Restitution,
    pub velocity: Velocity,

    // Pixel data for interaction with the sand simulation
    pub pixel: PixelComponent,

    // Image and sprite for rendering
    pub sprite: SpriteBundle,
}

impl DynamicPhysicsEntity {
    fn new(position: Vec2, image: &Image, handle: Handle<Image>, cell_type: CellType) -> Option<Self> {
        if let Some((pc, collider)) = process_image(image, cell_type) {
            return Some(Self {
                collider,
                rigidbody: RigidBody::Dynamic,
                mass: ColliderMassProperties::default(),
                restitution: Restitution::coefficient(0.5),
                velocity: Velocity::default(),
                pixel: pc,
                sprite: SpriteBundle {
                    texture: handle,
                    sprite: Sprite {
                        anchor: Anchor::BottomLeft,
                        ..Default::default()
                    },
                    transform: Transform::from_translation(position.extend(0.)),
                    ..Default::default()
                },
            })
        }
        None
    }
}

pub fn add_dpe(
    commands: &mut Commands,
    images: &Res<Assets<Image>>,
    position: Vec2,
    rigidbody_image: &Res<RigidBodyImageHandle>,
) {
    let image_handle = rigidbody_image.handle.clone().unwrap();
    let image = images.get(&image_handle).unwrap();

    let dpe = DynamicPhysicsEntity::new(position, image, image_handle.clone(), CellType::Stone);
    if let Some(dpe) = dpe {
        commands.spawn(dpe);
    }
}

/// Component that holds the pixel data for an entity
#[derive(Component)]
pub struct PixelComponent {
    pub size: UVec2,
    pub cells: Vec<Cell>,
}

impl PixelComponent {
    /// Creates a pixel component from an image with the given cell type for all cells
    pub fn from_image(image: &Image, cell_type: CellType) -> Self {
        let size = image.size();
        let cells: Vec<Cell> = image.data.chunks_exact(4).into_iter().map(|p| {
            Cell::with_cell_and_color(cell_type, [p[0], p[1], p[2], 255])
        }).collect();
        PixelComponent { size, cells }
    }
}

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

/// Create the collider and pixel component of an image
fn process_image(image: &Image, cell_type: CellType) -> Option<(PixelComponent, Collider)> {
    if let Some(collider) = create_convex_collider_from_values(image_valuemap(image).as_slice(), image.width() as f32, image.height() as f32) {
        return Some((
            PixelComponent::from_image(image, cell_type),
            collider
        ))
    }
    None
}

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
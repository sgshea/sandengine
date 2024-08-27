/// Type of rigid bodies that interact with the sand simulation

use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::{Collider, ReadMassProperties, Restitution, RigidBody, Velocity};

use crate::{particles::spawn_particle, pixel::{cell::{Cell, CellType, PhysicsType}, world::PixelWorld}, screen::Screen};

use super::collider_generation::create_convex_collider_from_values;

/// Bundle which includes physics properties along with the PixelComponent
#[derive(Bundle)]
pub struct DynamicPhysicsEntity {
    // Collider and Rigidbody of Rapier
    pub collider: Collider,
    pub rigidbody: RigidBody,

    // Modifiable properties of the rigidbody
    // TODO: custom generation of these properties based on the pixel data
    pub mass: ReadMassProperties,
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
                mass: ReadMassProperties::default(),
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
        commands.spawn(dpe).insert(
            StateScoped(Screen::Playing)
        );
    }
}

/// Component that holds the pixel data for an entity
#[derive(Component)]
pub struct PixelComponent {
    pub size: UVec2,
    pub cells: Vec<Cell>,

    // Location of filled cells in the world
    pub filled_tracker: Vec<IVec2>,
}

impl PixelComponent {
    /// Creates a pixel component from an image with the given cell type for all cells
    pub fn from_image(image: &Image, cell_type: CellType) -> Self {
        let size = image.size();
        let cells: Vec<Cell> = image.data.chunks_exact(4).into_iter().map(|p| {
            Cell::with_cell_and_color_rigidbody(cell_type, [p[0], p[1], p[2], 255])
        }).collect();
        PixelComponent { size, cells, filled_tracker: Vec::new() }
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

/// Fill the world with temporary cells based on the properties of the PixelComponents
pub fn fill_pixel_component(
    mut commands: Commands,
    mut sim: Query<&mut PixelWorld>,
    mut dpe: Query<(&Transform, &mut PixelComponent, &Velocity, &ReadMassProperties)>,
) {
    let world = &mut sim.single_mut();

    for (transform, mut pixel, velocity, mass) in &mut dpe {
        let angle = transform.rotation.to_euler(EulerRot::XYZ).2;
        // Translation of the dpe
        let translation = transform.translation.xy();

        // Iterate over and fill world with cells
        for y in 0..pixel.size.y {
            for x in 0..pixel.size.x {
                // Get the position of the cell in the world, accounting for rotation
                let pos = (translation + Vec2::new(x as f32, y as f32).rotate(Vec2::from_angle(angle))).round().as_ivec2();

                // Get the cell in the world
                let w_cell = world.get_cell(pos);

                // The the cell in the dpe
                let r_cell = pixel.cells[(y * pixel.size.x + x) as usize];
                // Make sure the physics type is correct (rigidbody)
                match r_cell.physics {
                    PhysicsType::RigidBody(_) => {
                        // Update the world cells based on the physics types, converting into particles

                        // If the cell will be converted into a particle or otherwise removed from the world or overwritten, set this flag to true
                        // Once we have finer logic we might be able to remove this but for now it's necessary:
                        // Because the dpe is rendered through image and not the internal pixel simulation, we need to ensure cells will not be overwritten when
                        // only a small amount of the component is inside a cell
                        let mut should_destroy_cell = false;
                        match PhysicsType::from(
                            // Get the type of the cell in the world, if it does not exist (dpe may be out of pixel world bounds), treat it as empty
                            if let Some(cell) = w_cell { cell.physics } else { PhysicsType::Empty },
                        ) {
                            PhysicsType::Empty => should_destroy_cell = true,
                            PhysicsType::SoftSolid(cell_type) | PhysicsType::Liquid(cell_type) => {
                                // Calculate the center of mass and the velocity at that point on the dpe
                                let center_of_mass = mass.local_center_of_mass + transform.translation.xy();
                                let velocity_at_point = velocity.linear_velocity_at_point(pos.as_vec2(), center_of_mass);
                                let normalized_velocity = velocity_at_point.normalize_or_zero() * (velocity_at_point.length() * mass.mass / 1000.);

                                spawn_particle(&mut commands, &Cell::from(cell_type), normalized_velocity, pos.as_vec2());
                                should_destroy_cell = true;
                            }
                            _ => {},
                        };

                        // Place the cell in the dpe into the world and keep track
                        if should_destroy_cell {
                            pixel.filled_tracker.push(pos);
                            world.set_cell_external(pos, Cell::object());
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}

/// Remove all PixelComponent cells that are marked as filled from the PixelWorld
pub fn unfill_pixel_component(
    mut sim: Query<&mut PixelWorld>,
    mut dpe: Query<&mut PixelComponent>,
) {
    let world = &mut sim.single_mut();

    for mut pixel in &mut dpe {
        while let Some(pos) = pixel.filled_tracker.pop() {
            world.set_cell_external(pos, Cell::default())
        }
    }
}
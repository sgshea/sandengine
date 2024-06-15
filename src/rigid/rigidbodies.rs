use bevy::{math::vec2, prelude::*};
use bevy_rapier2d::prelude::*;
use contour::ContourBuilder;
use geo::{SimplifyVwPreserve, TriangulateEarcut};
use strum::IntoEnumIterator;

use crate::{cell_types::StateType, pixel_plugin::PixelSimulation};

use super::RigidStorage;

// Generate colliders for every rigid body
pub fn generate_colliders(
    pixel_sim: Query<&mut PixelSimulation>,
    mut rigid_storage: ResMut<RigidStorage>,
    mut commands: Commands,
) {
    let world = &pixel_sim.single().world;
    let width = world.get_total_width();
    let height = world.get_total_height();

    let chunk_width = world.get_chunk_width();
    let chunk_height = world.get_chunk_height();

    let chunks = world.get_chunks();

    let origin_x = -width / 2;
    let origin_y = -height / 2;

    for (i, chunk) in chunks.iter().enumerate() {
        // Remove existing colliders
        cleanup_colliders(&mut rigid_storage, i, &mut commands);

        let mut colliders = vec![];
        // iterate over different types of state, later might want to specify collision types
        for state_type in StateType::iter() {
            if matches!(state_type, StateType::Empty(_)) {
                continue;
            }

            // Marching squares
            let contour_builder = ContourBuilder::new(chunk_width as usize, chunk_height as usize, false)
                                                    .x_origin(origin_x + (chunk.pos_x * chunk_width))
                                                    .y_origin(origin_y + (chunk.pos_y * chunk_height))
                                                    .x_step(1.0)
                                                    .y_step(1.0);
            let contours = contour_builder.contours(chunk.cells_as_floats().as_slice(), &[0.5]).expect("Failed to generate contours");

            for contour in contours {
                let geometry = contour.geometry().simplify_vw_preserve(&1.5);

                for poly in geometry {
                    let triangles = poly.earcut_triangles();
                    for triangle in triangles {
                        let collider = Collider::triangle(
                            vec2(triangle.0.x as f32, triangle.0.y as f32),
                            vec2(triangle.1.x as f32, triangle.1.y as f32),
                            vec2(triangle.2.x as f32, triangle.2.y as f32),
                        );

                        colliders.push((Vec2::ZERO, 0.0, collider));
                    }
                }
            }
        }
        if !colliders.is_empty() {
            let combined = Collider::compound(colliders);
            let id = commands.spawn(combined).id();
            rigid_storage.colliders[i] = Some(vec![id]);
        } else {
            rigid_storage.colliders[i] = None;
        }
    }
}

// Remove colliders inside a chunk
pub fn cleanup_colliders(
    rigid_storage: &mut ResMut<RigidStorage>,
    i: usize,
    commands: &mut Commands,
) {
    if let Some(colliders) = &rigid_storage.colliders[i] {
        for entity in colliders.iter() {
            commands.entity(*entity).despawn();
        }
    }
    rigid_storage.colliders[i] = None;
}
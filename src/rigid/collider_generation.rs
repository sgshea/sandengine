use bevy::{math::vec2, prelude::*};
use bevy_rapier2d::prelude::*;
use contour::ContourBuilder;
use geo::{Simplify, SimplifyVwPreserve, TriangulateEarcut};

use crate::pixel_plugin::PixelSimulation;

use super::RigidStorage;

// Generate colliders for each chunk
pub fn generate_colliders(
    pixel_sim: Query<&mut PixelSimulation>,
    mut rigid_storage: ResMut<RigidStorage>,
    mut commands: Commands,
) {
    let world = &pixel_sim.single().world;

    let chunk_width = world.get_chunk_width();
    let chunk_height = world.get_chunk_height();

    for (i, chunk) in world.get_chunks().iter().enumerate() {
        // Remove existing colliders
        cleanup_colliders(&mut rigid_storage, i, &mut commands);

        let mut colliders = vec![];

        // Apply the contour builder to the chunk
        // This uses the marching squares algorithm to create contours from the chunk data
        let contour_builder = ContourBuilder::new(chunk_width as usize, chunk_height as usize, false)
                                                // Adjust origin based on chunk position
                                                .x_origin(chunk.pos_x * chunk_width)
                                                .y_origin(chunk.pos_y * chunk_height)
                                                .x_step(1.0)
                                                .y_step(1.0);
        let contours = contour_builder.contours(chunk.cells_as_floats().as_slice(), &[0.5]).expect("Failed to generate contours");

        // Simplify and triangulate each contours
        for contour in contours {
            // simplify (Ramer-Douglas-Peucker algorithm) and simplify-vw-preserve (Visvalingam-Whyatt algorithm) are two candidates for simplifying the contours
            let geometry = contour.geometry().simplify_vw_preserve(&1.5);

            for poly in geometry {
                // Triangulate the polygon using the earcut algorithm and place into collider
                let triangles = convert_polygon_to_triangles(poly);
                for triangle in triangles.chunks(3) {
                    let collider = Collider::triangle(triangle[0], triangle[1], triangle[2]);
                    colliders.push((Vec2::ZERO, 0.0, collider));
                }
            }
        }
        if !colliders.is_empty() {
            // Combine all colliders into a single collider
            let combined = Collider::compound(colliders);
            let id = commands.spawn(combined).insert(ContactSkin(0.1)).id();
            rigid_storage.colliders[i] = Some(vec![id]);
        } else {
            rigid_storage.colliders[i] = None;
        }
    }
}

// Generate a single collider from values
// Can be used on an image for example
pub fn create_collider(values: &[f64], width: u32, height: u32) -> Option<Collider> {
    let mut colliders = vec![];

    // Apply the contour builder to the chunk
    // This uses the marching squares algorithm to create contours from the chunk data
    let contour_builder = ContourBuilder::new(width as usize, height as usize, false);
    let contours = contour_builder.contours(values, &[0.5]).expect("Failed to generate contour");

    // Simplify and triangulate each contours
    for contour in contours {
        // simplify (Ramer-Douglas-Peucker algorithm) and simplify-vw-preserve (Visvalingam-Whyatt algorithm) are two candidates for simplifying the contours
        // RDP is faster but VW is better at preserving the shape (creating better colliders)
        // Higher epsilon values will simplify more (remove more points)
        let geometry = contour.geometry().simplify_vw_preserve(&1.5);

        for poly in geometry {
            // Triangulate the polygon using the earcut algorithm and place into collider
            let triangles = convert_polygon_to_triangles(poly);
            for triangle in triangles.chunks(3) {
                let collider = Collider::triangle(triangle[0], triangle[1], triangle[2]);
                colliders.push((Vec2::ZERO, 0.0, collider));
            }
        }
    }
    match colliders.is_empty() {
        true => None,
        false => Some(Collider::compound(colliders)),
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

fn convert_polygon_to_triangles(polygon: geo::Polygon<f64>) -> Vec<Vec2> {
    let mut tris = Vec::new();
    let triangles = polygon.earcut_triangles();
    for triangle in triangles {
        tris.push(vec2(triangle.0.x as f32, triangle.0.y as f32));
        tris.push(vec2(triangle.1.x as f32, triangle.1.y as f32));
        tris.push(vec2(triangle.2.x as f32, triangle.2.y as f32));
    }
    tris
}
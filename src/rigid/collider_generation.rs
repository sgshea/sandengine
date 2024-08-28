use std::sync::mpsc::channel;

use bevy::{prelude::*, tasks::ComputeTaskPool};
use bevy_rapier2d::prelude::*;
use contour::{Contour, ContourBuilder};
use geo::{Area, CoordsIter, SimplifyVwPreserve};

use crate::{pixel::world::PixelWorld, screen::Screen};

use super::RigidStorage;

/// Generates colliders for the chunks in the pixel simulation
/// This function will regenerate a collider for each chunk in the simulation and add it to the rigid storage
/// If the chunk's dirty rectangle has not changed since the last frame, it will not generate a new collider
/// Chunk collider generate uses a polyline collider created through a simplified marching squares algorithm
pub fn chunk_collider_generation(
    pixel_sim: Query<&mut PixelWorld>,
    mut rigid_storage: ResMut<RigidStorage>,
    mut commands: Commands,
) {
    let world = &pixel_sim.single();

    let chunk_width = world.get_chunk_width();
    let chunk_height = world.get_chunk_height();

    // Make sure collider storage initialized with correct amount
    if rigid_storage.colliders.len() as u32 != world.chunk_amount.x * world.chunk_amount.y {
        rigid_storage.colliders.resize((world.chunk_amount.x * world.chunk_amount.y) as usize, None);
    }

    let chunks = world.get_chunks().into_iter().enumerate().map(|(i, chunk)| (i, chunk)).collect::<Vec<_>>();

    let (tx, rx) = channel::<(usize, Option<Vec<Collider>>)>();

    let mut update_counter = 0;
    ComputeTaskPool::get().scope(|scope| {
        for (index, chunk) in chunks {
            if !chunk.should_update() {
                continue;
            }
            update_counter += 1;
            let tx = tx.clone();
            scope.spawn(async move {
                // Apply the contour builder to the chunk
                // This uses the marching squares algorithm to create contours from the chunk data
                let contour_builder = ContourBuilder::new(chunk_width as usize, chunk_height as usize, false)
                                                        // Adjust origin based on chunk position
                                                        .x_origin(chunk.position.x * world.get_chunk_width() as i32)
                                                        .y_origin(chunk.position.y * world.get_chunk_height() as i32)
                                                        .x_step(1.0)
                                                        .y_step(1.0);
                let contours = contour_builder.contours(chunk.cells_as_floats().as_slice(), &[0.5]).expect("Failed to generate contours");

                // Create polyline colliders for each contour
                let mut colliders: Vec<Collider> = vec![];
                for contour in contours {
                    colliders.extend(create_polyline_colliders(&contour));
                }

                // Push colliders, if any were generated, to the storage
                if !colliders.is_empty() {
                    let mut id = vec![];
                    for collider in colliders {
                        id.push(collider);
                    }
                    tx.send((index, Some(id))).unwrap();
                } else {
                    tx.send((index, None)).unwrap();
                }
            });
        }
    });

    for _ in 0..update_counter {
        let (idx, colliders) = rx.recv().unwrap();
        // Despawn existing colliders
        if let Some(entities) = &rigid_storage.colliders[idx] {
            for e in entities {
                commands.entity(*e).despawn();
            }
        }
        // Place new colliders in by mapping to new entities
        match colliders {
            None => rigid_storage.colliders[idx] = None,
            Some(colliders) => {
                // map to entities
                let entities: Vec<Entity> = colliders.into_iter().map(|c| {
                    commands.spawn((c, StateScoped(Screen::Playing))).id()
                }).collect();
                rigid_storage.colliders[idx] = Some(entities)
            }
        }
    }
}

/// Create polyline colliders from a contour
fn create_polyline_colliders(contour: &Contour) -> Vec<Collider> {
    let geometry = contour.geometry().simplify_vw_preserve(&1.5);

    let mut edges = vec![];
    for poly in geometry {
        // Try to skip polygons that are too small
        if poly.unsigned_area() > 2.5 {
            let edge = poly.exterior_coords_iter().map(|p| Vec2::new(p.x as f32, p.y as f32));
            edges.push(Collider::polyline(edge.collect(), None));
        }
    }

    edges
}

/// Use rapier's convex_decomposition
fn create_convex_collider(contour: &Contour) -> Collider {
    let geometry = contour.geometry().simplify_vw_preserve(&1.5);
    let mut points: Vec<Vec2> = vec![];

    for poly in geometry.iter() {
        points.extend(poly.exterior_coords_iter().map(|p| Vec2::new(p.x as f32, p.y as f32)).collect::<Vec<_>>());
    }

    // We know that the points are sequentially ordered in the contour so we can create indices simply by counting to the next one
    let indices: Vec<[u32; 2]> = (0..points.len() - 1).map(|i| [i as u32, i as u32 + 1]).collect();

    Collider::convex_decomposition(&points, &indices)
}

/// Creates a single compound polyline collider from values
pub fn create_convex_collider_from_values(values: &[f64], width: f32, height: f32) -> Option<Collider> {

    let contour_builder = ContourBuilder::new(width as usize, height as usize, false);
    let contours = contour_builder.contours(values, &[0.5]).expect("Failed to generate contour");

    // Expect there to be only one contour
    let contour = contours.first();
    if contour.is_some() {
        return Some(create_convex_collider(contour.unwrap()))
    }
    None
}
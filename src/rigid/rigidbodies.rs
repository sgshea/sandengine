use std::ptr::eq;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use strum::IntoEnumIterator;

use crate::{cell_types::StateType, pixel_plugin::PixelSimulation, world::PixelWorld};

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

    let chunks = world.get_chunks();

    for (i, chunk) in chunks.iter().enumerate() {
        // Remove existing colliders
        cleanup_colliders(&mut rigid_storage, i, &mut commands);

        let min = world.chunk_to_world_coords(chunk.get_pos(), (0, 0));
        let max = world.chunk_to_world_coords(chunk.get_pos(), (chunk.width, chunk.height));

        let min = Vec2::new(min.0 as f32, min.1 as f32);
        let max = Vec2::new(max.0 as f32, max.1 as f32);

        let mut colliders: Vec<Entity> = Vec::new();
        // iterate over different types of state, later might want to specify collision types
        for state_type in StateType::iter() {
            if matches!(state_type, StateType::Empty(_)) {
                continue;
            }

            let blocks = march_edges(world, min, max, state_type);

            let theta = match state_type {
                StateType::HardSolid(_) => 1.0,
                _ => 2.0,
            };

            for block in &blocks {
                let block: Vec<_> = ramer_douglas_peucker(block, theta)
                    .into_iter()
                    .map(|pos| {
                        pos - Vec2::new(width as f32 / 2.0, height as f32 / 2.0)
                    }).collect();

                let collider = match state_type {
                    StateType::Empty(_) => panic!(),
                    StateType::SoftSolid(_) => commands.spawn(Collider::polyline(block, None)),
                    StateType::HardSolid(_) => commands.spawn(Collider::polyline(block, None)),
                    StateType::Liquid(_) => {
                        commands.spawn((
                            Collider::polyline(block, None),
                            Sensor,
                        ))
                    },
                    StateType::Gas(_) => {
                        commands.spawn((
                            Collider::polyline(block, None),
                            Sensor,
                        ))
                    },
                }.id();

                colliders.push(collider);
            }
        }
        rigid_storage.colliders[i] = Some(colliders);
    }
}

pub fn ramer_douglas_peucker(data: &[Vec2], epsilon: f32) -> Vec<Vec2> {
    let mut max_distance = 0.0;
    let mut index = 0;
    let end = data.len() - 1;

    for i in 1..end {
        let distance = perpendicular_distance(data[i], data[0], data[end]);
        if distance > max_distance {
            index = i;
            max_distance = distance;
        }
    }

    let mut results = vec![];

    if max_distance > epsilon {
        let mut recursive_results1 = ramer_douglas_peucker(&data[..index], epsilon);
        recursive_results1.remove(recursive_results1.len() - 1);
        let mut recursive_results2 = ramer_douglas_peucker(&data[index..], epsilon);

        // Build result
        results.append(&mut recursive_results1);
        results.append(&mut recursive_results2)
    } else {
        results = vec![data[0], data[end]];
    }

    results
}

pub fn perpendicular_distance(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let mut dx = line_end.x - line_start.x;
    let mut dy = line_end.y - line_start.y;

    // Normalise
    let magnitude = (dx.powf(2.0) + dy.powf(2.0)).powf(0.5);
    if magnitude > 0.0 {
        dx /= magnitude;
        dy /= magnitude;
    }

    let pvx = point.x - line_start.x;
    let pvy = point.y - line_start.y;

    // Get dot product (project pv onto normalized direction)
    let pvdot = dx * pvx + dy * pvy;

    // Scale line direction vector
    let dsx = pvdot * dx;
    let dsy = pvdot * dy;

    // Subtract this from pv
    let ax = pvx - dsx;
    let ay = pvy - dsy;

    (ax.powf(2.0) + ay.powf(2.0)).powf(0.5)
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

fn get_cell_state(world: &PixelWorld, state_type: StateType, x: i32, y: i32) -> i32 {
    let cell = world.get_cell(x, y);
    match cell {
        Some(cell) => {
            let cell_type = cell.get_state_type();
            match cell_type {
                StateType::Empty(_) => 0,
                _ => {
                    1
                }
            }
        }
        None => 0,
    }
}

// Based on https://github.com/shnewto/bevy_collider_gen
pub fn march_edges(
    world: &PixelWorld,
    low: Vec2,
    high: Vec2,
    state_type: StateType,
) -> Vec<Vec<Vec2>> {
    let mut edge_points: Vec<Vec2> = vec![];

    for x in low.x as i32..=high.x as i32 {
        for y in low.y as i32..=high.y as i32 {
            if get_cell_state(world, state_type, x, y) == 0 {
                continue;
            }

            let neighbors = [
                get_cell_state(world, state_type, x + 1, y),
                get_cell_state(world, state_type, x - 1, y),
                get_cell_state(world, state_type, x, y + 1),
                get_cell_state(world, state_type, x, y - 1),
            ];

            let (x, y) = (x as f32, y as f32);
            match neighbors {
                // Corners
                [1, 0, 0, 1] => {
                    edge_points.push(Vec2::new(x - 0.5, y - 0.5));
                    edge_points.push(Vec2::new(x - 0.5, y + 0.5));
                    edge_points.push(Vec2::new(x + 0.5, y + 0.5));
                }
                [1, 0, 1, 0] => {
                    edge_points.push(Vec2::new(x - 0.5, y + 0.5));
                    edge_points.push(Vec2::new(x - 0.5, y - 0.5));
                    edge_points.push(Vec2::new(x + 0.5, y - 0.5));
                }
                [0, 1, 0, 1] => {
                    edge_points.push(Vec2::new(x - 0.5, y + 0.5));
                    edge_points.push(Vec2::new(x + 0.5, y + 0.5));
                    edge_points.push(Vec2::new(x + 0.5, y - 0.5));
                }
                [0, 1, 1, 0] => {
                    edge_points.push(Vec2::new(x + 0.5, y + 0.5));
                    edge_points.push(Vec2::new(x + 0.5, y - 0.5));
                    edge_points.push(Vec2::new(x - 0.5, y - 0.5));
                }
                // Sides
                [1, 1, 1, 0] | [0, 0, 1, 0] => {
                    edge_points.push(Vec2::new(x - 0.5, y - 0.5));
                    edge_points.push(Vec2::new(x + 0.5, y - 0.5));
                }
                [1, 1, 0, 1] | [0, 0, 0, 1] => {
                    edge_points.push(Vec2::new(x - 0.5, y + 0.5));
                    edge_points.push(Vec2::new(x + 0.5, y + 0.5));
                }
                [1, 0, 1, 1] | [1, 0, 0, 0] => {
                    edge_points.push(Vec2::new(x - 0.5, y - 0.5));
                    edge_points.push(Vec2::new(x - 0.5, y + 0.5));
                }
                [0, 1, 1, 1] | [0, 1, 0, 0] => {
                    edge_points.push(Vec2::new(x + 0.5, y - 0.5));
                    edge_points.push(Vec2::new(x + 0.5, y + 0.5));
                }
                // Surrounded
                [1, 1, 1, 1] => continue,
                // Others
                _ => {
                    edge_points.push(Vec2::new(x + 0.5, y + 0.5));
                    edge_points.push(Vec2::new(x - 0.5, y + 0.5));
                    edge_points.push(Vec2::new(x - 0.5, y - 0.5));
                    edge_points.push(Vec2::new(x + 0.5, y - 0.5));
                }
            }
        }
    }

    points_to_drawing_order(&edge_points)
}

fn points_to_drawing_order(points: &[Vec2]) -> Vec<Vec<Vec2>> {
    let mut edge_points: Vec<Vec2> = points.to_vec();
    let mut in_drawing_order: Vec<Vec2> = vec![];
    let mut groups: Vec<Vec<Vec2>> = vec![];
    while !edge_points.is_empty() {
        if in_drawing_order.is_empty() {
            in_drawing_order.push(edge_points.remove(0));
        }

        let prev = *in_drawing_order.last().unwrap();

        let neighbor = edge_points
            .iter()
            .enumerate()
            .find(|(_, p)| prev.distance(**p) == 1.0);

        if let Some((i, _)) = neighbor {
            let next = edge_points.remove(i);
            in_drawing_order.push(next);
            continue;
        }

        if !in_drawing_order.is_empty() {
            groups.push(in_drawing_order.clone());
            in_drawing_order.clear()
        }
    }

    if !in_drawing_order.is_empty() {
        groups.push(in_drawing_order.clone());
    }

    groups
}

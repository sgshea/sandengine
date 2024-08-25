pub mod particle;

use bevy::prelude::*;
use particle::{Particle, PARTICLE_GRAVITY};

use crate::{pixel::{cell::{Cell, PhysicsType}, update_pixel_simulation, world::PixelWorld, PixelSimulation}, rigid::dynamic_entity::unfill_pixel_component};

/// Particle plugin
/// This plugin uses the same type of cells as the pixel plugin
/// However it is not based on cellular automata rules, instead the particles have non-integer positions as well as velocity
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update_particles
            .after(update_pixel_simulation)
            .before(unfill_pixel_component));
    }
}

pub fn spawn_particle(
    commands: &mut Commands,
    cell: &Cell,
    velocity: Vec2,
    position: Vec2,
) {
    commands.spawn((
        Particle::from_cell_with_velocity_position(cell, velocity),
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba_u8(cell.color[0], cell.color[1], cell.color[2], 255),
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            transform: Transform::from_translation(position.extend(1.)),
            ..Default::default()
        },
    ));
}

pub fn update_particles(
    mut commands: Commands,
    mut particles: Query<(&mut Particle, &mut Transform, Entity)>,
    mut pxl: Query<&mut PixelSimulation>,
) {
    let world = &mut pxl.single_mut().world;

    for (mut particle, mut transform, entity) in particles.iter_mut() {
        if apply_velocity(&mut particle, &mut transform, world) {
            commands.entity(entity).despawn();
        }
    }
}

/// Apply velocity, return true if particle was consumed and needs to be removed
fn apply_velocity(particle: &mut Particle, transform: &mut Transform, world: &mut PixelWorld) -> bool {
    match particle.physics {
        PhysicsType::Gas(_) => particle.velocity.y += PARTICLE_GRAVITY,
        _ => particle.velocity.y -= PARTICLE_GRAVITY,
    };

    let vector_length = particle.velocity.length();
    if vector_length < 0.5 {
        world.set_cell_external(transform.translation.truncate().as_ivec2(), Cell::from(particle.clone()));
        return true;
    }

    // Normalize velocity and convert to i32 for line
    let normalized_velocity = (particle.velocity / vector_length).as_ivec2();

    let cur_pos = transform.translation.truncate().round().as_ivec2();

    // Find first intersection
    for i in 0..=(vector_length as i32).abs() {
        let next_pos = cur_pos + normalized_velocity * i;
        if is_occupied(world, next_pos) {
            // Hit occupied cell, stop and consume into last position
            let last_pos = cur_pos + normalized_velocity * (i - 1);
            // If last position also occupied, reverse velocity and try next frame
            if is_occupied(world, last_pos) {
                particle.velocity = -particle.velocity * 0.90;
            } else {
                world.set_cell_external(last_pos, Cell::from(particle.clone()));
                return true;
            }
        }
    }

    // No intersection found
    transform.translation += particle.velocity.extend(0.);
    false
}

fn is_occupied(
    world: &PixelWorld,
    pos: IVec2,
) -> bool {
    if let Some(cell) = world.get_cell(pos) {
        matches!(cell.physics, PhysicsType::HardSolid(_))
    } else {
        false
    }
}
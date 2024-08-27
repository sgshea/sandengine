pub mod particle;

use bevy::{prelude::*, render::view::RenderLayers};
use particle::{Particle, PARTICLE_GRAVITY};

use crate::{pixel::{cell::{Cell, PhysicsType}, update_pixel_simulation, world::PixelWorld}, rigid::dynamic_entity::unfill_pixel_component, screen::Screen};

/// Particle plugin
/// This plugin uses the same type of cells as the pixel plugin
/// However it is not based on cellular automata rules, instead the particles have non-integer positions as well as velocity
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update_particles
            .after(update_pixel_simulation)
            .before(unfill_pixel_component)
            .run_if(in_state(Screen::Playing)));
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
            transform: Transform::from_translation(position.extend(3.)),
            ..Default::default()
        },
        RenderLayers::layer(3),
    ));
}

pub fn update_particles(
    mut commands: Commands,
    mut particles: Query<(&mut Particle, &mut Transform, Entity)>,
    mut pxl: Query<&mut PixelWorld>,
) {
    let world = &mut pxl.single_mut();

    for (mut particle, mut transform, entity) in particles.iter_mut() {
        if apply_velocity(&mut particle, &mut transform, world) {
            commands.entity(entity).despawn();
        }
    }
}

/// Apply velocity, return true if particle was consumed and needs to be removed
fn apply_velocity(particle: &mut Particle, transform: &mut Transform, world: &mut PixelWorld) -> bool {
    if particle.velocity.length() < 0.4 {
        world.set_cell_external(transform.translation.xy().as_ivec2(), Cell::from(particle.clone()));
        return true;
    }

    match particle.physics {
        PhysicsType::Gas(_) => particle.velocity.y += PARTICLE_GRAVITY,
        _ => particle.velocity.y -= PARTICLE_GRAVITY,
    };

    let deltav = particle.velocity;

    let steps = (deltav.x.abs() + deltav.y.abs()).sqrt() as usize + 1;
    for s in 0..steps {
        let n = (s + 1) as f32 / steps as f32;
        transform.translation += n * deltav.extend(0.) * 0.90;

        if let Some(cell) = world.get_cell(transform.translation.truncate().as_ivec2()) {
            match cell.physics {
                PhysicsType::Empty => {
                    if s == steps - 1 {
                        return false;
                    }
                },
                _ => {
                    if s > 0 {
                        // Turn into cell
                        world.set_cell_external(transform.translation.truncate().as_ivec2(), Cell::from(particle.clone()));
                        return true
                    } else {
                        // Extra velocity in order to get out of whatever area we are in
                        particle.velocity.y = if matches!(particle.physics, PhysicsType::Gas(_)) { -1. } else { 1. };
                        particle.velocity.x = if particle.velocity.x >= 0. { -0.4 } else { 0.4 };
                        break;
                    }
                }
            };
        }
        particle.velocity *= 0.80;
    }
    false
}
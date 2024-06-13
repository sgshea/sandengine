// Creates colliders for every rigid body and stores per chunk

mod rigidbodies;

use bevy::prelude::*;
use rigidbodies::generate_colliders;

use super::CHUNKS;

pub struct SandEngineRigidPlugin;

impl Plugin for SandEngineRigidPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RigidStorage::default())
            .add_systems(Update, generate_colliders);

    }
}

// RigidStorage is a resource that stores a vector for each chunk that contains the entities of the colliders in that chunk
#[derive(Resource)]
pub struct RigidStorage {
    pub colliders: Vec<Option<Vec<Entity>>>,
}

impl Default for RigidStorage {
    fn default() -> Self {
        Self {
            colliders: vec![None; CHUNKS.0 as usize * CHUNKS.1 as usize]
        }
    }
}
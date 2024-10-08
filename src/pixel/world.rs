use std::sync::mpsc::channel;

use bevy::{
    math::{IVec2, UVec2},
    prelude::Component,
    tasks::ComputeTaskPool,
    utils::{hashbrown::HashMap, syncunsafecell::SyncUnsafeCell},
};

use super::{
    cell::Cell,
    chunk::PixelChunk,
    chunk_handler::SimulationChunkContext,
    geometry_helpers::{BoundRect, DIRECTIONS},
};

use rand::prelude::SliceRandom;

// Pixel world component which holds the chunks, as well as general information
#[derive(Component)]
pub struct PixelWorld {
    pub chunk_size: UVec2,
    pub world_size: UVec2,
    pub chunk_amount: UVec2,

    pub chunks: HashMap<IVec2, PixelChunk>,

    iteration: u32,
}

impl PixelWorld {
    // Create a new pixel world based on the total size and how many chunks it should be subdivided into
    pub fn new(world_size: UVec2, chunk_amount: UVec2) -> Self {
        let mut new_world = PixelWorld {
            chunk_amount,
            world_size,
            chunk_size: world_size / chunk_amount,
            chunks: HashMap::new(),
            iteration: 0,
        };

        // create chunks
        for x in 0..new_world.chunk_amount.x {
            for y in 0..new_world.chunk_amount.y {
                new_world.create_chunk(x as i32, y as i32);
            }
        }

        new_world
    }

    // Return position of chunk and dirty rect
    pub fn get_chunk_dirty_rects(&self) -> Vec<(IVec2, BoundRect)> {
        self.chunks
            .iter()
            .map(|(key, val)| (*key, val.current_dirty_rect))
            .collect()
    }

    fn create_chunk(&mut self, x: i32, y: i32) {
        let chunk = PixelChunk::new(self.chunk_size, IVec2 { x, y });
        self.chunks.insert(IVec2 { x, y }, chunk);
    }

    pub fn get_chunk_width(&self) -> u32 {
        self.chunk_size.x
    }

    pub fn get_chunk_height(&self) -> u32 {
        self.chunk_size.y
    }

    pub fn get_chunks(&self) -> Vec<&PixelChunk> {
        self.chunks.values().collect()
    }

    fn chunk(&self, position: IVec2) -> Option<&PixelChunk> {
        self.chunks.get(&position)
    }

    // Returns chunk data to render if the chunk has updated, None if not
    pub fn should_render_data(&self, position: IVec2) -> Option<Vec<u8>> {
        let chunk = self.chunk(position);
        if let Some(c) = chunk {
            if c.should_update() {
                return Some(c.render_chunk());
            }
        }
        None
    }

    /// Gets all the chunks that should update and returns their positions
    fn all_chunk_pos_should_update(&self) -> Vec<IVec2> {
        self.chunks
            .iter()
            .filter(|&(_, chunk)| chunk.should_update())
            .map(|(&pos, _)| pos)
            .collect()
    }

    fn chunk_mut(&mut self, position: IVec2) -> Option<&mut PixelChunk> {
        self.chunks.get_mut(&position)
    }

    // Finds the chunk of a given world coordinate
    pub fn cell_to_chunk_position(chunk_size: UVec2, position: IVec2) -> IVec2 {
        position.div_euclid(chunk_size.as_ivec2())
    }

    // Finds the cell position inside of a chunk given a world cell coordinate
    pub fn cell_to_position_in_chunk(chunk_size: UVec2, position: IVec2) -> IVec2 {
        let chunk_position = Self::cell_to_chunk_position(chunk_size, position);

        position - chunk_position * chunk_size.x as i32
    }

    // Get a cell based on it's world coordinate
    pub fn get_cell(&self, position: IVec2) -> Option<Cell> {
        let chunk = self.chunk(Self::cell_to_chunk_position(self.chunk_size, position))?;

        let local = Self::cell_to_position_in_chunk(self.chunk_size, position);
        Some(chunk.get_cell(local))
    }

    // Finds if the cell is inside a dirty rectangle of a chunk
    pub fn cell_inside_dirty(&self, position: IVec2) -> bool {
        let chunk = self.chunk(Self::cell_to_chunk_position(self.chunk_size, position));

        if let Some(chunk) = chunk {
            let local = Self::cell_to_position_in_chunk(self.chunk_size, position);

            return chunk.current_dirty_rect.contains(&local);
        }
        false
    }

    // Sets the value of a cell in this chunk, if it exists.
    // Makes sure that the chunk is marked as dirty if it wasn't already.
    pub fn set_cell_external(&mut self, position: IVec2, cell: Cell) {
        let chunk_size = self.chunk_size.clone();
        let Some(chunk) = self.chunk_mut(Self::cell_to_chunk_position(chunk_size, position)) else {
            return;
        };

        let local = Self::cell_to_position_in_chunk(chunk_size, position);
        chunk.set_cell(local.x, local.y, cell);

        chunk.render_override = 3;
    }

    // Main update function
    pub fn update(&mut self) {
        let all_pos = self.all_chunk_pos_should_update();
        let chunk_size = self.chunk_size;

        // Channel for recieving updates to the dirty rects
        let (tx, rx) = channel::<HashMap<IVec2, Vec<IVec2>>>();

        let mut unsafe_cell_chunks: HashMap<IVec2, &SyncUnsafeCell<PixelChunk>> = HashMap::new();
        for pos in all_pos.clone() {
            if let Some(ch) = self.chunk_mut(pos) {
                ch.commit_cells_unupdated();

                // Converting to UnsafeCell for sending to threads
                // # Safety (of this particular part)
                // a &mut PixelChunk is the same as a &UnsafeCell<PixelChunk>
                let unsafe_chunk: &SyncUnsafeCell<PixelChunk> = unsafe { std::mem::transmute(ch) };
                unsafe_cell_chunks.insert(pos, unsafe_chunk);
            }
        }

        // Shuffling the order of updates to avoid bias
        // It makes large amounts of movements between chunks feel a bit more natural instead of favoring one direction of movement
        let mut rng = rand::thread_rng();
        let mut iterations = [(0, 0), (1, 0), (0, 1), (1, 1)];
        iterations.shuffle(&mut rng);

        // Count how many chunks get updated so that we know how many dirty rect updates will be recieved through the channel
        let mut update_counter = 0;
        ComputeTaskPool::get().scope(|scope| {
            for iter in iterations {
                all_pos.iter().for_each(|pos| {
                    // Calculate if this position should update for this iteration
                    let xx = (pos.x + iter.0) % 2 == 0;
                    let yy = (pos.y + iter.1) % 2 == 0;
                    if xx && yy && self.chunk(*pos).is_some_and(|c| c.should_update()) {
                        update_counter += 1;
                        let unsafe_chunks = unsafe_cell_chunks.clone();
                        let tx = tx.clone();
                        scope.spawn(async move {
                            // Send result of this calculation through the channel
                            tx.send({
                                // Collect the surrounding and center chunk
                                let arr = DIRECTIONS
                                    .map(|dir| {
                                        let chunk = unsafe_chunks.get(&(*pos + dir));
                                        match chunk {
                                            Some(c) => Some(*c),
                                            None => None,
                                        }
                                    })
                                    .into_iter()
                                    .collect::<Vec<Option<&SyncUnsafeCell<PixelChunk>>>>();

                                // Simulate a chunk by creating the context for simulation
                                let mut scc = SimulationChunkContext::new(*pos, arr, chunk_size);
                                scc.simulate()
                            })
                            .unwrap();
                        });
                    }
                });
            }
        });

        // Recieve through the channel and merge all of the dirty rect updates
        let mut dirty_rect_updates: HashMap<IVec2, Vec<IVec2>> = HashMap::new();
        for _ in 0..update_counter {
            let new_update = rx.recv().unwrap();
            for (position, cells) in new_update {
                if let Some(existing) = dirty_rect_updates.get_mut(&position) {
                    existing.extend(cells);
                } else {
                    dirty_rect_updates.insert(position, cells);
                }
            }
        }

        // Apply dirty rect updates
        for (position, cells) in dirty_rect_updates {
            if let Some(ch) = self.chunk_mut(position) {
                ch.swap_rects();
                ch.construct_dirty_rect(&cells);
            }
        }
        self.iteration += 1;
    }
}

use std::cell::UnsafeCell;

use bevy::{math::IVec2, tasks::ComputeTaskPool, utils::hashbrown::HashMap};

use crate::{pixel::chunk_handler::SimulationChunkContext, CHUNK_SIZE};

use super::{cell::Cell, chunk::PixelChunk, geometry_helpers::{BoundRect, DIRECTIONS}};

pub struct PixelWorld {
    c_height: i32,
    c_width: i32,

    chunks_x: i32,
    chunks_y: i32,

    pub chunks: HashMap<IVec2, PixelChunk>,

    iteration: u32,
}

impl PixelWorld {

    pub fn new(t_width: i32, t_height: i32, chunks_x: i32, chunks_y: i32) -> Self {
        let mut new_world = PixelWorld {
            c_height: t_height / chunks_x,
            c_width: t_width / chunks_y,
            chunks_x,
            chunks_y,
            chunks: HashMap::new(),
            iteration: 0
        };

        // create chunks
        for x in 0..new_world.chunks_x {
            for y in 0..new_world.chunks_y {
                new_world.create_chunk(x, y);
            }
        }

        new_world
    }

    // Return position of chunk and dirty rect
    pub fn get_chunk_dirty_rects(&self) -> Vec<(IVec2, BoundRect)> {
        self.chunks.iter()
            .map(|(key, val)| {
                (
                    *key,
                    val.current_dirty_rect
                )
            })
            .collect()
    }

    fn create_chunk(&mut self, x: i32, y: i32) {
        let chunk = PixelChunk::new(self.c_width, self.c_height, x, y);
        self.chunks.insert(IVec2 { x, y }, chunk);
    }

    pub fn get_chunk_width(&self) -> i32 {
        self.c_width
    }

    pub fn get_chunk_height(&self) -> i32 {
        self.c_height
    }

    pub fn get_chunks(&self) -> Vec<&PixelChunk> {
        self.chunks.values().collect()
    }

    fn chunk(&self, position: IVec2) -> Option<&PixelChunk> {
        self.chunks.get(&position)
    }

    fn chunk_mut(&mut self, position: IVec2) -> Option<&mut PixelChunk> {
        self.chunks.get_mut(&position)
    }

    pub fn cell_to_chunk_position(position: IVec2) -> IVec2 {
        position.div_euclid(CHUNK_SIZE)
    }

    pub fn cell_to_position_in_chunk(position: IVec2) -> IVec2 {
        let chunk_position = Self::cell_to_chunk_position(position);

        position - chunk_position * CHUNK_SIZE.x
    }

    pub fn get_cell(&self, position: IVec2) -> Option<Cell> {
        let chunk = self.chunk(Self::cell_to_chunk_position(position))?;

        let local = Self::cell_to_position_in_chunk(position);
        Some(chunk.get_cell(local))
    }

    pub fn cell_inside_dirty(&self, position: IVec2) -> bool {
        let chunk = self.chunk(Self::cell_to_chunk_position(position));

        if let Some(chunk) = chunk {
            let local = Self::cell_to_position_in_chunk(position);

            return chunk.current_dirty_rect.contains(&local)
        }
        false
    }

    pub fn set_cell(&mut self, position: IVec2, cell: Cell) {
        let Some(chunk) = self.chunk_mut(Self::cell_to_chunk_position(position)) else {
            return;
        };

        let local = Self::cell_to_position_in_chunk(position);
        chunk.set_cell(local.x, local.y, cell);
    }

    // Main update function
    pub fn update(&mut self) {
        let taskpool = ComputeTaskPool::get();

        let all_pos: Vec<IVec2> = self.chunks.keys().map(|pos| *pos).collect::<Vec<IVec2>>();

        // Contains all the new updates, used to contruct dirty rects for next frame
        let mut dirty_rect_updates: HashMap<IVec2, Vec<IVec2>> = HashMap::new();

        for pos in all_pos.clone() {
            if let Some(ch) = self.chunk_mut(pos) {
                ch.commit_cells_unupdated();
            }
        }

        let iterations = [(0, 0), (1, 0), (0, 1), (1, 1)];

        for iter in iterations {
            all_pos.iter().for_each(|pos| {
                let xx = (pos.x + iter.0) % 2 == 0;
                let yy = (pos.y + iter.1) % 2 == 0;
                if xx && yy && self.chunk(*pos).is_some_and(|c| c.should_update()) {
                    let new_updates = {
                        let arr = DIRECTIONS
                        .map(|dir|{
                            let chunk = self.chunk_mut(*pos + dir);
                            match chunk {
                                Some(c) => {
                                    let cell_chunk: &UnsafeCell<PixelChunk> = unsafe { std::mem::transmute(c) };
                                    Some(cell_chunk)
                                },
                                None => None,
                            }
                        }).into_iter().collect::<Vec<Option<&UnsafeCell<PixelChunk>>>>();

                        // Simulate a chunk by creating the context and push into the taskpool for simulation
                        let mut scc = SimulationChunkContext::new(
                            *pos,
                            arr.try_into().unwrap(),
                        );
                        taskpool.scope(|s| {
                            s.spawn(async move {
                                scc.simulate()
                            })
                        })
                    };

                    // Push new updates into dirty rect updates
                    for i in new_updates {
                        for (position, cells) in i {
                            if let Some(existing) = dirty_rect_updates.get_mut(&position) {
                                existing.extend(cells);
                            } else {
                                dirty_rect_updates.insert(position, cells);
                            }
                        }
                    }
                }
            });
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

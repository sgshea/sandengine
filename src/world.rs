use rayon::prelude::*;
use bevy::utils::hashbrown::HashMap;

use crate::{cell::Cell, chunk::{PixelChunk, SplitChunk}, cworker::ChunkWorker};
use rand::seq::SliceRandom;

pub struct PixelWorld {
    c_height: i32,
    c_width: i32,

    chunks_x: i32,
    chunks_y: i32,

    pub chunks_lookup: HashMap<(i32, i32), PixelChunk>,

    iteration: u32,
}

impl PixelWorld {

    pub fn new(t_width: i32, t_height: i32, chunks_x: i32, chunks_y: i32) -> Self {
        let mut new_world = PixelWorld {
            c_height: t_height / chunks_x,
            c_width: t_width / chunks_y,
            chunks_x,
            chunks_y,
            chunks_lookup: HashMap::new(),
            iteration: 0
        };

        // create chunks
        for x in 0..chunks_x {
            for y in 0..chunks_y {
                new_world.create_chunk(x, y);
            }
        }

        new_world
    }

    // Get locations of all chunks that are awake
    pub fn get_awake_chunk_locs(&self) -> Vec<(i32, i32)> {
        self.chunks_lookup.values().filter_map(|chunk| {
            if chunk.awake {
                Some((chunk.pos_x, chunk.pos_y))
            } else {
                None
            }
        }).collect()
    }

    pub fn get_chunk_location(&self, x: i32, y: i32) -> (i32, i32) {
        (x / self.c_width, y / self.c_height)
    }

    pub fn get_chunk(&self, x: i32, y: i32) -> &PixelChunk {
        let (cx, cy) = self.get_chunk_location(x, y);
        self.chunks_lookup.get(&(cx, cy)).unwrap()
    }

    fn create_chunk(&mut self, x: i32, y: i32) {
        let chunk = PixelChunk::new(self.c_width, self.c_height, x, y);
        self.chunks_lookup.insert((x, y), chunk);
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.c_width * self.chunks_x && y >= 0 && y < self.c_height * self.chunks_y
    }

    pub fn chunk_to_world_coords(&self, chunk_pos: (i32, i32), cell_pos: (i32, i32)) -> (i32, i32) {
        (chunk_pos.0 * self.c_width + cell_pos.0, chunk_pos.1 * self.c_height + cell_pos.1)
    }

    pub fn inside_chunk(&self, chunk: &PixelChunk, world_coord: (i32, i32)) -> bool {
        (chunk.pos_x, chunk.pos_y) == self.get_chunk_location(world_coord.0, world_coord.1)
    }

    pub fn chunk_exists_at_world_coord(&self, x: i32, y: i32) -> bool {
        self.chunks_lookup.contains_key(&self.get_chunk_location(x, y))
    }

    pub fn get_cell(&self, x: i32, y: i32) -> Option<&Cell> {
        if x < 0 || y < 0 || x >= self.get_total_width() || y >= self.get_total_height() {
            return None;
        }
        match self.chunks_lookup.get(&self.get_chunk_location(x, y)) {
            Some(chunk) => Some(chunk.get_cell_2d(x, y)),
            None => None,
        }
    }

    pub fn set_cell(&mut self, x: i32, y: i32, cell: Cell) {
        match self.chunks_lookup.get_mut(&self.get_chunk_location(x, y)) {
            Some(chunk) => chunk.set_cell(x, y, cell),
            None => (),
        }
    }

    pub fn get_total_width(&self) -> i32 {
        self.c_width * self.chunks_x
    }

    pub fn get_total_height(&self) -> i32 {
        self.c_height * self.chunks_y
    }

    pub fn get_chunk_width(&self) -> i32 {
        self.c_width
    }

    pub fn get_chunk_height(&self) -> i32 {
        self.c_height
    }

    pub fn get_chunks(&self) -> Vec<&PixelChunk> {
        self.chunks_lookup.values().collect()
    }

    // Update cells
    pub fn update(&mut self) {
        let all_pos = self.chunks_lookup.keys().map(|pos| *pos).collect::<Vec<(i32, i32)>>();

        // Shuffle iterations each time
        let mut iterations = [(0, 0), (1, 0), (0, 1), (1, 1)];
        let rng = &mut rand::thread_rng();
        iterations.shuffle(rng);

        for (x, y) in iterations.iter() {
            let iteration_x_y = (*x, *y);
            let chunks = &mut self.chunks_lookup;
            let mut current_references: HashMap<(i32, i32), SplitChunk> = HashMap::new();
            get_chunk_references(chunks, &mut current_references, iteration_x_y);

            let mut workers: Vec<ChunkWorker> = Vec::new();
            all_pos.iter().for_each(|pos| {
                let x = (pos.0 + iteration_x_y.0) % 2 == 0;
                let y = (pos.1 + iteration_x_y.1) % 2 == 0;
                if x && y {
                    // Lifetime explanation:
                    // we can borrow on each iteration because no references to the hashmap items are kept
                    // the ChunkWorker removes the center chunk from the hashmap, so we can borrow the hashmap again
                    // the needed parts of the SplitChunk are also removed from the hashmap using mem::take and similarly not kept in the hashmaps
                    workers.push(ChunkWorker::new_from_chunk_ref(pos, &mut current_references, self.iteration % 2 == 0));
                }
            });
            workers.iter_mut().for_each(|worker| {
                worker.update();
            });
        }
        // reset updated_at and swap buffers
        self.chunks_lookup.values_mut().par_bridge().for_each(|chunk| {
            // swap buffers and reset updated
            chunk.cells.iter_mut().for_each(|cell| {
                cell.updated = 0;
            });
            // chunk.cells = chunk.next_cells.clone();
        });
        self.iteration += 1;
    }
}

// Turns all chunks into split chunks
pub(crate) fn get_chunk_references<'a>(
    chunks: &'a mut HashMap<(i32, i32), PixelChunk>,
    current: &mut HashMap<(i32, i32), SplitChunk<'a>>,
    iteration_x_y: (i32, i32),
) {
    chunks.iter_mut().for_each(|(pos, chunk)| {
        let x = (pos.0 + iteration_x_y.0) % 2 == 0;
        let y = (pos.1 + iteration_x_y.1) % 2 == 0;

        match (x, y) {
            (true, true) => {
                // for the 'center' chunks, because SplitChunk references the whole chunk, we just insert into the current references
                current.insert(*pos, SplitChunk::from_chunk(chunk));
            },
            (false, true) => {
                let cur = SplitChunk::from_chunk_side(chunk);
                current.insert(*pos, cur);
            },
            (true, false) => {
                let cur = SplitChunk::from_chunk_vert(chunk);
                current.insert(*pos, cur);
            },
            (false, false) => {
                let cur = SplitChunk::from_chunk_corners(chunk);
                current.insert(*pos, cur);
            },
        }
    });
}
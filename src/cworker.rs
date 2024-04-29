

use std::mem;

use bevy::utils::hashbrown::HashMap;

use crate::{cell::Cell, cell_types::CellType, chunk::{PixelChunk, SplitChunk}};

pub struct ChunkWorker<'a> {
    chunk: &'a mut PixelChunk,
    surrounding_chunks: HashMap<(i32, i32), Option<Vec<&'a mut Cell>>>,
}

impl<'a> ChunkWorker<'a> {
    pub fn new_from_chunk_ref(pos: &(i32, i32), chunks: &mut HashMap<(i32, i32), SplitChunk<'a>>) -> Self {
        // get center
        let chunk = match chunks.remove(pos).unwrap() {
            SplitChunk::Entire(chunk) => chunk,
            _ => panic!("Expected entire chunk for center"),
        };
        let surrounding_chunks = get_surrounding_chunks(chunks, pos.0, pos.1);

        Self {
            chunk,
            surrounding_chunks,
        }
    }

    pub fn update(&mut self) {
        // do basic falling over center chunk for testing
        let center_chunk = &mut self.chunk;

        for i in 0..center_chunk.cells.len() {
            let x = i % center_chunk.width as usize;
            let y = i / center_chunk.width as usize;

            if center_chunk.cells[i].get_type() == CellType::Empty {
                continue;
            }

            if y > 0 {
                let below_idx = center_chunk.get_index(x as i32, (y - 1) as i32);
                if center_chunk.cells[below_idx].get_type() == CellType::Empty {
                    center_chunk.cells.swap(i, below_idx);
                }
            }

        }
    }
}

pub fn get_surrounding_chunks<'a>(
    chunks: &mut HashMap<(i32, i32), SplitChunk<'a>>,
    x: i32,
    y: i32,
) -> HashMap<(i32, i32), Option<Vec<&'a mut Cell>>> {
    let mut surrounding_chunks = HashMap::new();
    for i in -1..2 {
        for j in -1..2 {
            let pos = (x + i, y + j);
            if let Some(chunk) = chunks.get_mut(&pos) {
                match chunk {
                    SplitChunk::Entire(_) => { continue; },
                    SplitChunk::TopBottom(chunk) => {
                        if j == 1 {
                            surrounding_chunks.insert(pos, mem::take(&mut chunk[0]));
                        } else {
                            surrounding_chunks.insert(pos, mem::take(&mut chunk[1]));
                        }
                    },
                    SplitChunk::LeftRight(chunk) => {
                        if i == 1 {
                            surrounding_chunks.insert(pos, mem::take(&mut chunk[0]));
                        } else {
                            surrounding_chunks.insert(pos, mem::take(&mut chunk[1]));
                        }
                    },
                    SplitChunk::Corners(_) => {
                        // unimplemented
                        continue;
                    },
                };
            }
        }
    }

    surrounding_chunks
}
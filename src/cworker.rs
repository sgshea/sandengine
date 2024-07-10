use std::{fmt::Debug, mem};

use bevy::{math::Vec2, utils::hashbrown::HashMap};
use rand::Rng;

use crate::{cell::Cell, cell_types::{CellType, DirectionType, StateType}, chunk::{PixelChunk, SplitChunk}};

pub struct ChunkWorker<'a> {
    chunk: &'a mut PixelChunk,
    surrounding: HashMap<(i32, i32), Option<Vec<&'a mut Cell>>>,
    iter_dir: bool,
}

impl<'a> ChunkWorker<'a> {
    pub fn new_from_chunk_ref(pos: &(i32, i32), current: &mut HashMap<(i32, i32), SplitChunk<'a>>, iter_dir: bool) -> Self {
        // get center
        let chunk = match current.remove(pos).unwrap() {
            SplitChunk::Entire(chunk) => chunk,
            _ => panic!("Expected entire chunk for center"),
        };
        let surrounding= get_surrounding_chunks(current, pos.0, pos.1);

        Self {
            chunk,
            surrounding,
            iter_dir,
        }
    }

    pub fn update(&mut self) {
        for y in 0..self.chunk.height {
            if self.iter_dir {
                for x in 0..self.chunk.width {
                    self.update_cell(x, y);
                }
            } else {
                for x in (0..self.chunk.width).rev() {
                    self.update_cell(x, y);
                }
            }
        }
    }

    fn update_cell(&mut self, x: i32, y: i32) {
        let state_type = self.chunk.cells[get_index(x, y, self.chunk.width as i32)].get_state_type();
        let updated = self.chunk.cells[get_index(x, y, self.chunk.width as i32)].updated;
        if updated == 1 {
            return;
        }
        match state_type {
            StateType::Empty(_) => {
                // do nothing
                return;
            },
            StateType::SoftSolid(_) => {
                let idx = self.get_worker_index(x, y);
                let down_empty = self.get_other_cell(&idx, DirectionType::DOWN).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_)));
                let down_left_empty = self.get_other_cell(&idx, DirectionType::DOWN_LEFT).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_)));
                let down_right_empty = self.get_other_cell(&idx, DirectionType::DOWN_RIGHT).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_)));

                if down_empty && (!(down_left_empty || down_right_empty) || rand::thread_rng().gen_range(0..10) != 0) {
                    self.downward_fall(&idx);
                } else {
                    self.down_side(&idx);
                }
            }
            StateType::Liquid(_) => {
                let idx = self.get_worker_index(x, y);

                let down_empty = self.get_other_cell(&idx, DirectionType::DOWN).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_)));
                let left_empty = self.get_other_cell(&idx, DirectionType::LEFT).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_)));
                let right_empty = self.get_other_cell(&idx, DirectionType::RIGHT).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_)));
                if down_empty && (!(left_empty || right_empty) || rand::thread_rng().gen_bool(0.95)) {
                    self.downward_fall(&idx);
                } else {
                    self.sideways(&idx);
                }
            }
            StateType::Gas(_) => {
                let idx = self.get_worker_index(x, y);
                self.apply_force(&idx, DirectionType::UP, 1.);
                if self.apply_velocity(&idx) {
                    return;
                }
                if self.sideways(&idx) {
                    return;
                }
            }
            _ => {
                // do nothing
            }
        }
    }

    fn swap_cells(&mut self, c1: &WorkerIndex, c2: &WorkerIndex) -> bool {
        let (x1, y1) = c1.chunk_rel;

        // c1 should always be in the center chunk
        assert!(x1 == 0 && y1 == 0);

        match c2.chunk_rel {
            (0, 0) => {
                // If the cell has been updated, but is empty, give a small chance to still swap
                if self.chunk.cells[c1.idx].updated == 1 ||
                 (self.chunk.cells[c2.idx].updated == 1 && !matches!(self.chunk.cells[c2.idx].get_state_type(), StateType::Empty(_))
                 && rand::thread_rng().gen_bool(0.1)) {
                    return false;
                }
                self.chunk.cells.swap(c1.idx, c2.idx);

                // mark as updated
                self.chunk.cells[c2.idx].updated = 1;
                self.chunk.cells[c1.idx].updated = 1;
            },
            (x, y) => {
                let chunk = self.surrounding.get_mut(&(x, y)).unwrap();

                // if the cell has been updated, we cannot swap
                if self.chunk.cells[c1.idx].updated == 1 || chunk.as_ref().unwrap()[c2.idx].updated == 1 {
                    return false;
                }

                let c1_c = self.chunk.cells[c1.idx].clone();
                self.chunk.cells[c1.idx] = chunk.as_mut().unwrap()[c2.idx].clone();
                *chunk.as_mut().unwrap()[c2.idx] = c1_c;

                // mark as updated
                chunk.as_mut().unwrap()[c2.idx].updated = 1;
                self.chunk.cells[c1.idx].updated = 1;
            },
        }
        true
    }

    // Gets the index of a relative chunk and index within that chunk
    fn get_worker_index(&self, x: i32, y: i32) -> WorkerIndex {
        if x >= 0 && x < self.chunk.width && y >= 0 && y < self.chunk.height {
        return WorkerIndex {
            chunk_rel: (0, 0),
            idx: get_index(x, y, self.chunk.width as i32),
            x,
            y,
        };
        } else {
            // if negative, we are dealing with a chunk to the left or below
            let x_c = if x < 0 { -1 } else if x >= self.chunk.width as i32 { 1 } else { 0 };
            let y_c = if y < 0 { -1 } else if y >= self.chunk.height as i32 { 1 } else { 0 };

            let width = self.chunk.width;
            let width_2 = width / 2;
            let width_4 = width / 4;

            let (n_x, n_y, w) = match (x_c, y_c) {
                (0, 1) => (x, y % width, width),
                (0, -1) => (x, (y + width_2), width),
                (1, 0) => (x % width_2, y, width_2),
                (-1, 0) => ((x + width_2), y, width_2),

                (-1, 1) => ((x % width_4) + width_2, y % width, width_4),
                (1, 1) => (x % width_4, y % width, width_4),
                (-1, -1) => ((x % width_4) + width_2, (y + width_2), width_2),
                (1, -1) => (x % width_4, (y + width_2), width_2),
                _ => panic!("Invalid chunk relative position"),
            };

            // println!("{} {} ({}, {})", get_index(x, y, w), get_index(x, y, self.chunk.width as i32), x, y);

            WorkerIndex {
                chunk_rel: (x_c, y_c),
                idx: get_index(n_x, n_y, w),
                x,
                y,
            }
        }
    }

    fn get_cell(&self, x: i32, y: i32) -> Option<&Cell> {
        let idx = self.get_worker_index(x, y);
        match idx.chunk_rel {
            (0, 0) => Some(&self.chunk.cells[idx.idx]),
            (x, y) => {
                match self.surrounding.get(&(x, y)) {
                    None => None,
                    Some(chunk) => {
                        match chunk {
                            None => None,
                            Some(chunk) => {
                                // let cell = &chunk[other_idx.idx];
                                // println!("{} {} ({}, {}), ({:?})", idx.idx, other_idx.idx, other_idx.x, other_idx.y, cell.get_type());
                                Some(&chunk[idx.idx])
                            }
                        }
                    }
                }
            },
        }
    }

    fn get_other_cell(&self, idx: &WorkerIndex, dir: DirectionType) -> Option<&Cell> {
        let (x, y) = (idx.x, idx.y);
        let other_idx = dir.get_tuple_direction();
        let other_idx = (x + other_idx.0, y + other_idx.1);
        let other_idx = self.get_worker_index(other_idx.0, other_idx.1);
        match other_idx.chunk_rel {
            (0, 0) => Some(&self.chunk.cells[other_idx.idx]),
            (x, y) => {
                match self.surrounding.get(&(x, y)) {
                    None => None,
                    Some(chunk) => {
                        match chunk {
                            None => None,
                            Some(chunk) => {
                                // println!("{} {} ({}, {})", idx.idx, other_idx.idx, other_idx.x, other_idx.y);
                                // chunk sizes
                                // println!("{} {}", chunk.len(), chunk.len());
                                Some(&chunk[other_idx.idx])
                            }
                        }
                    }
                }
            },
        }
    }

    fn downward_fall(&mut self, idx: &WorkerIndex) -> bool {
        // are few below clear
        let empty_below = (0..4).all(|i| {
            let other_cell = self.get_cell(idx.x, idx.y - 2 - i);
            other_cell.is_some() && matches!(other_cell.unwrap().get_state_type(), StateType::Empty(_))
        });

        if empty_below {
            self.apply_gravity(idx);
            return self.apply_velocity(idx);
        } else {
            // move 1 or 2 steps down
            if rand::thread_rng().gen_bool(0.5) && self.get_cell(idx.x, idx.y - 2).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_))) {
                let new_idx = self.get_worker_index(idx.x, idx.y - 2);
                return self.swap_cells(idx, &new_idx);
            } else {
                let new_idx = self.get_worker_index(idx.x, idx.y - 1);
                return self.swap_cells(idx, &new_idx);
            }
        }
    }

    fn down_side(&mut self, idx: &WorkerIndex) -> bool {
        let down_left_empty = self.get_other_cell(&idx, DirectionType::DOWN_LEFT).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_)));
        let down_right_empty = self.get_other_cell(&idx, DirectionType::DOWN_RIGHT).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_)));
        let above_empty = self.get_other_cell(&idx, DirectionType::UP).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_)));

        // covered cells less likely to move down to sides
        if above_empty || rand::thread_rng().gen_bool(0.5) {
            if down_left_empty && down_right_empty {
                // choose 50/50
                let move_left = rand::thread_rng().gen_bool(0.5);
                let new_idx = if move_left {
                    self.get_worker_index(idx.x - 1, idx.y - 1)
                } else {
                    self.get_worker_index(idx.x + 1, idx.y - 1)
                };
                return self.swap_cells(idx, &new_idx);
            } else if down_left_empty {
                // chance to move down by 2
                if rand::thread_rng().gen_bool(0.5) && self.get_cell(idx.x - 1, idx.y - 2).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_))) {
                    let new_idx = self.get_worker_index(idx.x - 1, idx.y - 2);
                    return self.swap_cells(idx, &new_idx);
                } else {
                    let new_idx = self.get_worker_index(idx.x - 1, idx.y - 1);
                    return self.swap_cells(idx, &new_idx);
                }
            } else if down_right_empty {
                // chance to move down by 2
                if rand::thread_rng().gen_bool(0.5) && self.get_cell(idx.x + 1, idx.y - 2).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_))) {
                    let new_idx = self.get_worker_index(idx.x + 1, idx.y - 2);
                    return self.swap_cells(idx, &new_idx);
                } else {
                    let new_idx = self.get_worker_index(idx.x + 1, idx.y - 1);
                    return self.swap_cells(idx, &new_idx);
                }
            }
        }
        false
    }

    fn sideways(&mut self, idx: &WorkerIndex) -> bool {
        let left_empty = self.get_other_cell(&idx, DirectionType::LEFT).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_)));
        let right_empty = self.get_other_cell(&idx, DirectionType::RIGHT).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_)));

        if left_empty && right_empty {
            // choose 50/50
            let move_left = rand::thread_rng().gen_bool(0.5);
            // Try each, if swap fails, try the other direction
            if move_left && self.swap_cells(idx, &self.get_worker_index(idx.x - 1, idx.y)) {
                return true;
            } else if !move_left && self.swap_cells(idx, &self.get_worker_index(idx.x + 1, idx.y)) {
                return true;
            } return false;
        } else if left_empty {
            if rand::thread_rng().gen_bool(0.5) && self.get_cell(idx.x - 2, idx.y).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_))) {
                let new_idx = self.get_worker_index(idx.x - 2, idx.y);
                return self.swap_cells(idx, &new_idx);
            } else {
                let new_idx = self.get_worker_index(idx.x - 1, idx.y);
                return self.swap_cells(idx, &new_idx);
            }
        } else if right_empty {
            if rand::thread_rng().gen_bool(0.5) && self.get_cell(idx.x + 2, idx.y).is_some_and(|t| matches!(t.get_state_type(), StateType::Empty(_))) {
                let new_idx = self.get_worker_index(idx.x + 2, idx.y);
                return self.swap_cells(idx, &new_idx);
            } else {
                let new_idx = self.get_worker_index(idx.x + 1, idx.y);
                return self.swap_cells(idx, &new_idx);
            }
        }
        false
    }

    // Applies a force in direction with amount
    fn apply_force(&mut self, source: &WorkerIndex, direction: DirectionType, amount: f32) {
        // check direction exists
        let cell_in_direction = match self.get_other_cell(source, direction) {
            Some(cell) => cell.clone(),
            None => {
                Cell::new(CellType::Stone, DirectionType::NONE)
            }
        };
        let other_density = cell_in_direction.get_density();
        let max_speed = self.chunk.width as f32;

        let cell = &mut self.chunk.cells[source.idx];
        // Clamp current velocity
        cell.velocity = cell.velocity.clamp(Vec2::new(-max_speed, -max_speed), Vec2::new(max_speed, max_speed));

        let cell_density = cell.get_density();
        if other_density < cell_density {
            let limit = 5.;
            match direction {
                DirectionType::LEFT => {
                    if cell.velocity.x > -limit {
                        cell.velocity.x -= amount;
                    }
                },
                DirectionType::RIGHT => {
                    if cell.velocity.x < limit {
                        cell.velocity.x += amount;
                    }
                },
                DirectionType::UP => {
                    if cell.velocity.y < limit {
                        cell.velocity.y -= amount;
                    }
                },
                DirectionType::DOWN => {
                    if cell.velocity.y < limit {
                        cell.velocity.y += amount;
                    }
                },
                _ => {},
            }
        } else {
            // deflection into adjacent direction when hitting a wall or hitting ground
            let other_velocity = cell_in_direction.velocity;
            match direction {
                // hitting wall (left or right)
                DirectionType::LEFT | DirectionType::RIGHT => {
                    if other_velocity.x.abs() < 0.5 {
                        // deflection into y direction based on cell movement type
                        if cell.get_movement().contains(DirectionType::DOWN) {
                            cell.velocity.y -= cell.velocity.x / 3.;
                        } else if cell.get_movement().contains(DirectionType::UP) {
                            cell.velocity.y += cell.velocity.x / 3.;
                        }
                        cell.velocity.x = 0.;
                    }
                },
                // hitting ground or ceiling
                DirectionType::DOWN | DirectionType::UP => {
                    if other_velocity.y.abs() < 0.5 {
                        // deflection into x direction
                        if cell.velocity.x == 0. {
                            // 50% chance to go left or right
                            if rand::thread_rng().gen_bool(0.5) {
                                cell.velocity.x += cell.velocity.y / 3.;
                            } else {
                                cell.velocity.x -= cell.velocity.y / 3.;
                            }
                        } else {
                            if cell.velocity.x < 0. {
                                cell.velocity.x -= (cell.velocity.y / 3.).abs();
                            } else {
                                cell.velocity.x += (cell.velocity.y / 3.).abs();
                            }
                        }
                        cell.velocity.y = 0.;
                    }
                },
                _ => {},
            }
        }
    }

    // Applies gravity to the cell
    // Shortcuts to apply_force
    fn apply_gravity(&mut self, idx: &WorkerIndex) {
        self.apply_force(idx, DirectionType::DOWN, 1.);
    }

    fn apply_velocity(&mut self, idx: &WorkerIndex) -> bool {
        let cell = &mut self.chunk.cells[idx.idx];
        let cell_density = cell.get_density();

        let vector_length = cell.velocity.length();

        // No significant velocity
        if vector_length < 0.5 {
            return false;
        }

        // clamp to half chunk length (assumes square chunks)
        // ensuring that it does not try to move outside of what the worker has access to
        let max_velocity = self.chunk.width as f32 / 2.;
        cell.velocity.x = cell.velocity.x.clamp(-max_velocity, max_velocity);
        cell.velocity.y = cell.velocity.y.clamp(-max_velocity, max_velocity);

        // reset x dir
        if cell.get_type() == CellType::Sand && cell.velocity.x.abs() < 1. {
            cell.velocity.x = 0.;
        }

        let (f_x, f_y) = (cell.velocity.x / vector_length, cell.velocity.y / vector_length);

        // No significant force
        if f_x == 0. && f_y == 0. {
            return false;
        }

        // Moving elements to furthest position possible
        let (mut max_x, mut max_y) = (idx.x as f32, idx.y as f32);
        let (x, y) = (idx.x as f32, idx.y as f32);
        let mut drag = 1.0;
        for i in 1..=vector_length.round() as i32 {
            // calculate index
            let (x, y) = ((x as f32 - (f_x * i as f32)).round() as i32, (y as f32 - (f_y * i as f32)).round() as i32);

            // trying to move here
            let other_cell = self.get_cell(x, y);

            // cell is none or solid, cannot move futher
            if other_cell.is_none() || matches!(other_cell.unwrap().get_state_type(), StateType::HardSolid(_)) {
                if i == 1 || other_cell.is_none() {
                    // immediately stoped
                    let cell = &mut self.chunk.cells[idx.idx];
                    cell.velocity = Vec2::ZERO;
                    return false;
                } else {
                    if max_x != idx.x as f32 || max_y != idx.y as f32 {
                        // move to max_x, max_y
                        let cell = &mut self.chunk.cells[idx.idx];
                        cell.velocity *= drag;
                        let new_idx = self.get_worker_index(max_x as i32, max_y as i32);
                        return self.swap_cells(idx, &new_idx);
                    } else {
                        // stop
                        let cell = &mut self.chunk.cells[idx.idx];
                        cell.velocity = Vec2::ZERO;
                        return false;
                    }
                }
            } else {
                if other_cell.unwrap().get_density() < cell_density {
                    // new furthest position
                    drag = 0.7;
                    (max_x, max_y) = (x as f32, y as f32);
                }
            }

            // No solid cells found and at maximum length
            if i == vector_length.round() as i32 {
                if max_x != idx.x as f32 || max_y != idx.y as f32 {
                    // move to max_x, max_y
                    let cell = &mut self.chunk.cells[idx.idx];
                    cell.velocity *= drag;
                    let new_idx = self.get_worker_index(max_x as i32, max_y as i32);
                    return self.swap_cells(idx, &new_idx);
                } else {
                    // stop
                    let cell = &mut self.chunk.cells[idx.idx];
                    cell.velocity = Vec2::ZERO;
                    return false;
                }
            }
        }
        false
    }
}

struct WorkerIndex {
    chunk_rel: (i32, i32),
    idx: usize, // idx within chunk

    // original x and y
    x: i32,
    y: i32,
}

impl Debug for WorkerIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WorkerIndex {{ chunk_rel: {:?}, idx: {}, x: {}, y: {} }}", self.chunk_rel, self.idx, self.x, self.y)
    }
}

fn get_index(x: i32, y: i32, width: i32) -> usize {
    (y * width + x) as usize
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
            let pos_rel = (i, j);
            if let Some(chunk) = chunks.get_mut(&pos) {
                match chunk {
                    SplitChunk::Entire(_) => { continue; },
                    SplitChunk::TopBottom(chunk) => {
                        if j == 1 {
                            surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[0]));
                        } else {
                            surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[1]));
                        }
                    },
                    SplitChunk::LeftRight(chunk) => {
                        if i == 1 {
                            surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[0]));
                        } else {
                            surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[1]));
                        }
                    },
                    SplitChunk::Corners(chunk) => {
                        match pos_rel {
                            (1, 1) => {
                                surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[0]));
                            },
                            (1, -1) => {
                                surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[2]));
                            },
                            (-1, -1) => {
                                surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[3]));
                            },
                            (-1, 1) => {
                                surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[1]));
                            },
                            _ => { continue; },
                        }
                    },
                };
            }
        }
    }

    surrounding_chunks
}

#[cfg(test)]
mod tests {
    use crate::world::{get_chunk_references, PixelWorld};

    use super::*;

    // Test getting indices in surrounding chunks math
    // Representation:
    // 240	241	242	243	244	245	246	247	248	249	250	251	252	253	254	255
    // 224	225	226	227	228	229	230	231	232	233	234	235	236	237	238	239
    // 208	209	210	211	212	213	214	215	216	217	218	219	220	221	222	223
    // 192	193	194	195	196	197	198	199	200	201	202	203	204	205	206	207
    // 176	177	178	179	180	181	182	183	184	185	186	187	188	189	190	191
    // 160	161	162	163	164	165	166	167	168	169	170	171	172	173	174	175
    // 144	145	146	147	148	149	150	151	152	153	154	155	156	157	158	159
    // 128	129	130	131	132	133	134	135	136	137	138	139	140	141	142	143
    // 112	113	114	115	116	117	118	119	120	121	122	123	124	125	126	127
    // 96	97	98	99	100	101	102	103	104	105	106	107	108	109	110	111
    // 80	81	82	83	84	85	86	87	88	89	90	91	92	93	94	95
    // 64	65	66	67	68	69	70	71	72	73	74	75	76	77	78	79
    // 48	49	50	51	52	53	54	55	56	57	58	59	60	61	62	63
    // 32	33	34	35	36	37	38	39	40	41	42	43	44	45	46	47
    // 16	17	18	19	20	21	22	23	24	25	26	27	28	29	30	31
    // 0	1	2	3	4	5	6	7	8	9	10	11	12	13	14	15
    // A single corner would be:
    // 56	57	58	59	60	61	62	63
    // 48	49	50	51	52	53	54	55
    // 40	41	42	43	44	45	46	47
    // 32	33	34	35	36	37	38	39
    // 24	25	26	27	28	29	30	31
    // 16	17	18	19	20	21	22	23
    // 8	9	10	11	12	13	14	15
    // 0	1	2	3	4	5	6	7
    #[test]
    fn test_surrounding_chunks_worker_indices() {
        // Create test world
        // Each chunk is 16x16
        let mut world = PixelWorld::new(64, 64, 4, 4);

        let chunks = &mut world.chunks_lookup;
        let mut current_references: HashMap<(i32, i32), SplitChunk> = HashMap::new();
        get_chunk_references(chunks, &mut current_references, (1, 1));

        let test_worker = ChunkWorker::new_from_chunk_ref(&(1, 1), &mut current_references, true);

        let pos = (test_worker.chunk.pos_x, test_worker.chunk.pos_y);
        assert_eq!(pos, (1, 1));
        // Bottom left corner chunk
        let pos_1= test_worker.get_worker_index(-1, -1);
        assert_eq!(pos_1.chunk_rel, (-1, -1));
        assert_eq!(pos_1.idx, 63);

        // Bottom right corner chunk
        let pos_2 = test_worker.get_worker_index(16, -1);
        assert_eq!(pos_2.chunk_rel, (1, -1));
        assert_eq!(pos_2.idx, 56);

        // Top left corner chunk
        let pos_3 = test_worker.get_worker_index(-1, 16);
        assert_eq!(pos_3.chunk_rel, (-1, 1));
        assert_eq!(pos_3.idx, 7);

        // Top right corner chunk
        let pos_4 = test_worker.get_worker_index(16, 16);
        assert_eq!(pos_4.chunk_rel, (1, 1));
        assert_eq!(pos_4.idx, 0);
    }
}
use std::{collections::{HashSet, HashMap}, io::{self, Write, BufWriter, Read, BufReader}};

use rand::{rngs::ThreadRng, random};
use tiny_skia::{Pixmap, Paint, LineJoin, Stroke, LineCap, PathBuilder, Rect, Transform, Color, BlendMode, PixmapPaint, FilterQuality};

use crate::{pool::{Pool, NodeId}, dijkstra::{DijkstraPad, Distance}, grid::Direction};



pub struct MaskedGrid {
    pub pool: Pool<(usize, usize)>,
    pub mask: Box<dyn Fn(usize, usize) -> bool>,
    pub width: usize,
    pub height: usize,
    pub cell_grid: HashMap<(usize, usize), NodeId>,
}

impl PartialEq for MaskedGrid {
    fn eq(&self, other: &Self) -> bool {
        for row in 0..self.height {
            for col in 0..self.width {
                if (self.mask)(row, col) != (other.mask)(row, col) {
                    return false;
                }
            }
        }
        self.pool == other.pool && self.width == other.width && self.height == other.height && self.cell_grid == other.cell_grid
    }
}

#[derive(Debug)]
pub enum GridReadError {
    IoError(std::io::Error),
    NotEnoughBytes,
    TooManyBytes,
    InvalidNewsGrid(NewsGridError),
}

#[derive(Debug)]
pub enum NewsGridError {
    UnrequitedConnection {
        linked: (usize, usize),
        unlinked: (usize, usize),
        direction: Direction,
    },
    ConnectedOutOfBounds {
        cell: (usize, usize),
        direction: Direction,
    },
    ConnectedOutOfMask {
        linked: (usize, usize),
        missing: (usize, usize),
        direction: Direction,
    }
}

impl From<std::io::Error> for GridReadError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<NewsGridError> for GridReadError {
    fn from(value: NewsGridError) -> Self {
        GridReadError::InvalidNewsGrid(value)
    }
}

impl MaskedGrid {

    pub fn new_unmasked(width: usize, height: usize) -> Self {
        Self::new(width, height, Box::new(|_, _| true))
    }

    pub fn new(width: usize, height: usize, mask: Box<dyn Fn(usize, usize) -> bool>) -> Self {
        let mut pool: Pool<(usize, usize)> = Pool::new();
        let mut cell_grid: HashMap<(usize, usize), NodeId> = HashMap::new();
        // First populate the pool and grids. No connections are made yet.
        for row in 0..height {
            for col in 0..width {
                if !mask(row, col) {
                    continue;
                }
                let id = pool.new_node(|_| (row, col));
                cell_grid.insert((row, col), id);
            }
        }

        // Stitch together the nodes
        for row in 0..height {
            for col in 0..width {
                if !mask(row, col) {
                    continue;
                }
                let left_col = col.checked_sub(1);
                let right_col = col.checked_add(1);
                let above_row = row.checked_sub(1);
                let below_row = row.checked_add(1);
                let center_id = *cell_grid.get(&(row, col)).unwrap(); // Can always as cell_grid has an entry for every position contained in mask
                let left_cell = left_col.map(|lcol| cell_grid.get(&(row, lcol))).flatten().cloned();
                let right_cell = right_col.map(|rcol| cell_grid.get(&(row, rcol))).flatten().cloned();
                let above_cell = above_row.map(|arow| cell_grid.get(&(arow, col))).flatten().cloned();
                let below_cell = below_row.map(|brow| cell_grid.get(&(brow, col))).flatten().cloned();

                if let Some(left_id) = left_cell {
                    pool.make_adjacent(center_id, left_id, true);
                }
                if let Some(right_id) = right_cell {
                    pool.make_adjacent(center_id, right_id, true);
                }
                if let Some(above_id) = above_cell {
                    pool.make_adjacent(center_id, above_id, true);
                }
                if let Some(below_id) = below_cell {
                    pool.make_adjacent(center_id, below_id, true);
                }
            }
        }

        assert!(pool.is_adjacently_connected(), "Given mask comprises of disjoint parts!");

        Self {
            pool, mask: Box::new(mask), width, height, cell_grid
        }
    }

    pub fn total_cells(&self) -> usize {
        self.pool.nodes.len()
    }

    pub fn aldous_broder(&mut self, rng: &mut ThreadRng) {
        self.pool.aldous_broder(rng);
        return;
        
        /*// Pick a random starting cell
        let mut cell = Uniform::from(0..self.total_cells()).sample(rng);
        let mut unvisited = self.total_cells() - 1;
        while unvisited > 0 {
            let neighbors: Vec<NodeId> = self.pool.neighborhood_of(cell).into_iter().collect();
            let neighbor = neighbors[rng.gen_range(0..neighbors.len())];
            if self.pool.get(neighbor).links.is_empty() {
                self.pool.link_cells(cell, neighbor, true);
                unvisited-=1;
            }
            cell = neighbor;
        }*/
    }

    pub fn hunt_and_kill(&mut self, rng: &mut ThreadRng) {
        self.pool.hunt_and_kill(rng);
        return;
    }

    pub fn scan_frontier(&self, visited: &HashSet<NodeId>) -> Option<(NodeId, NodeId)> {
        for node in self.pool.nodes.iter().filter(|n| !visited.contains(&n.id)) {
            for wall in self.pool.walls_of(node.id) {
                if visited.contains(&wall) {
                    return Some((node.id, wall));
                }
            }
        }

        return None;
    }

    pub fn get_id_at(&self, row: usize, col: usize) -> Option<NodeId> {
        self.cell_grid.get(&(row, col)).cloned()
    }

    pub fn is_linked(&self, here: NodeId, there: NodeId) -> bool {
        self.pool.is_linked(here, there)
    }

    pub fn is_linked_at(&self, row_here: usize, col_here: usize, row_there: usize, col_there: usize) -> bool {
        (self.mask)(row_here, col_here)
        && (self.mask)(row_there, col_there)
        && self.is_linked(*self.cell_grid.get(&(row_here, col_here)).unwrap()
                         , *self.cell_grid.get(&(row_there, col_there)).unwrap())
    }

    pub fn is_h_wall(&self, row: usize, col: usize) -> bool {
        if col == self.width {
            return false;
        }
        if row == 0 {
            return (self.mask)(row, col);
        }
        if row == self.height {
            return (self.mask)(row - 1, col);
        }
        
        let present_above = (self.mask)(row - 1, col);
        let present_center = (self.mask)(row, col);
        match (present_above, present_center) {
            (true, true) => {
                return !self.is_linked_at(row, col, row - 1, col);
            },
            (true, false) => { return true; },
            (false, true) => { return true; },
            (false, false) => { return false; },
        }
    }

    pub fn is_v_wall(&self, row: usize, col: usize) -> bool {
        if row == self.width {
            return false;
        }
        if col == 0 {
            return (self.mask)(row, col);
        }
        if col == self.width {
            return (self.mask)(row, col - 1);
        }

        let present_left = (self.mask)(row, col - 1);
        let present_center = (self.mask)(row, col);
        match (present_left, present_center) {
            (true, true) => {
                return !self.is_linked_at(row, col, row, col - 1);
            },
            (true, false) => { return true; },
            (false, true) => { return true; },
            (false, false) => { return false; },
        }
    }
    
    pub fn print_image(&self, cell_size: usize, padding: usize, draw_walls: bool, paint_function: impl Fn(NodeId) -> Paint<'static>, icons: Vec<(NodeId, Pixmap)>) -> Pixmap {
        let image_width = self.width * cell_size + 2 * padding;
        let image_height = self.height * cell_size + 2 * padding;
        let mut pixmap = Pixmap::new(image_width as u32, image_height as u32).unwrap();

        let black = {
            let mut paint = Paint::default();
            paint.set_color_rgba8(0,0,0, u8::MAX);
            paint.anti_alias = true;
            paint
        };

        let stroke = {
            let mut stroke = Stroke::default();
            stroke.width = 3.0;
            stroke.line_cap = LineCap::Round;
            stroke.line_join = LineJoin::Round;
            stroke
        };

        // Paint the interiors
        for row in 0..self.height {
            for col in 0..self.width {
                if !(self.mask)(row, col) {
                    continue;
                }
                
                let top = (row * cell_size + padding) as f32;
                let bottom = ((row + 1) * cell_size + padding) as f32;
                let left = (col * cell_size + padding) as f32;
                let right = ((col + 1) * cell_size + padding) as f32;

                pixmap.fill_rect(Rect::from_ltrb(
                    left,
                    top,
                    right,
                    bottom,
                ).unwrap(), &paint_function(*self.cell_grid.get(&(row, col)).unwrap()), Transform::identity(), None);

                for (id, icon) in &icons {
                    if self.cell_grid.get(&(row, col)) == Some(id) {
                        let icon_size = icon.width().min(icon.height()) as f32;
                        pixmap.draw_pixmap(0, 0, icon.as_ref(), &{ 
                            let mut p = PixmapPaint::default();
                            p.blend_mode = BlendMode::SourceOver;
                            p.quality = FilterQuality::Bicubic;
                            p
                        }, Transform::identity().pre_scale(
                            (right - left) * 0.6 / icon_size,
                            (bottom - top) * 0.6 / icon_size,
                        ).post_translate(left + (padding as f32), top + (padding as f32)), None);
                    }
                }
            }
        }

        if draw_walls {
            let path = {
                let mut pb = PathBuilder::new();
    
                for row in 0..=self.height {
                    for col in 0..=self.width {
                        let top = (row * cell_size + padding) as f32;
                        let bottom = ((row + 1) * cell_size + padding) as f32;
                        let left = (col * cell_size + padding) as f32;
                        let right = ((col + 1) * cell_size + padding) as f32;
    
                        if self.is_h_wall(row, col) {
                            pb.move_to(left, top);
                            pb.line_to(right, top);
                        }
                        if self.is_v_wall(row, col) {
                            pb.move_to(left, top);
                            pb.line_to(left, bottom);
                        }
                    }
                }
                
                pb.finish().unwrap()
            };

            pixmap.stroke_path(&path, &black, &stroke, Transform::identity(), None);
        }

        return pixmap;
    }


    pub fn print_image_distances(&self, cell_size: usize, padding: usize, start_node: NodeId, draw_walls: bool, color_function: impl Fn(f64) -> Color) -> Pixmap {
        let distances = DijkstraPad::new(&self.pool, start_node).perform();
        let max_finite_distance = distances.pool.payloads().map(|d| {
            match d {
                Distance::Finite(dist) => *dist,
                Distance::Infinite => 0,
            }
        }).max().unwrap_or(0) as f64;

        if max_finite_distance == 0.0 {
            self.print_image(cell_size, padding, draw_walls, |_| {
                let mut p = Paint::default();
                p.set_color_rgba8(u8::MAX, u8::MAX, u8::MAX, u8::MAX);
                p
            }, vec![])
        } else {
            self.print_image(cell_size, padding, draw_walls, |node_id| {
                let dist = distances.pool.get(node_id).payload.as_finite().unwrap_or(0) as f64;
                let normalized_distance = dist / max_finite_distance;
                let mut p = Paint::default();
                p.set_color(color_function(normalized_distance));
                p
            }, vec![])
        }
    }

    fn mask_rectangle(top: usize, left: usize, bottom: usize, right: usize) -> HashSet<(usize, usize)> {
        (top..bottom).flat_map(|row| (left..right).map(move |col| (row, col))).collect()
    }

    pub fn cell_to_byte(&self, row: usize, col: usize) -> u8 {
        if self.cell_grid.contains_key(&(row, col)) {
            let north: u8 = if row > 0 && self.is_linked_at(row, col, row - 1, col) {
                0b1000
            } else { 0 };

            let west: u8 = if col > 0 && self.is_linked_at(row, col, row, col - 1) {
                0b0010
            } else { 0 };

            let east: u8 = if col < self.width - 1 && self.is_linked_at(row, col, row, col + 1) {
                0b0100
            } else { 0 };

            let south: u8 = if row < self.height - 1 && self.is_linked_at(row, col, row + 1, col) {
                0b0001
            } else { 0 };

            north | east | west | south
        } else { 0 }
    }

    pub fn render_to_mask(&self, cell_size: usize, wall_half_width: usize, shortcut_probability: f32, shortcut_thickness: usize) -> HashSet<(usize, usize)> {
        let grid_spacing = cell_size + 2 * wall_half_width;
        let mut result_mask: HashSet<(usize, usize)> = HashSet::new();
        for row in 0..self.height {
            for col in 0..self.width {
                let grid_top = row * grid_spacing;
                let grid_left = col * grid_spacing;
                let grid_bottom = (row + 1) * grid_spacing;
                let grid_right = (col + 1) * grid_spacing;
                let cell_top = grid_top + wall_half_width;
                let cell_left = grid_left + wall_half_width;
                let cell_bottom = grid_bottom - wall_half_width;
                let cell_right = grid_right - wall_half_width;
                let shortcut_top = grid_top + grid_spacing / 2 - shortcut_thickness / 2;
                let shortcut_left = grid_left + grid_spacing / 2 - shortcut_thickness / 2;
                let shortcut_bottom = shortcut_top + shortcut_thickness;
                let shortcut_right = shortcut_left + shortcut_thickness;

                if (self.mask)(row, col) {
                    result_mask.extend(Self::mask_rectangle(cell_top, cell_left, cell_bottom, cell_right));
                } else {
                    continue;
                }
                
                if !self.is_h_wall(row, col) {
                    result_mask.extend(Self::mask_rectangle(grid_top, cell_left, cell_top, cell_right));
                } 

                if !self.is_h_wall(row + 1, col) {
                    result_mask.extend(Self::mask_rectangle(cell_bottom, cell_left, grid_bottom, cell_right));
                } else if self.cell_grid.contains_key(&(row, col)) && self.cell_grid.contains_key(&(row + 1, col)) {
                    if random::<f32>() < shortcut_probability {
                        result_mask.extend(Self::mask_rectangle(cell_bottom, shortcut_left, cell_top + grid_spacing, shortcut_right));
                    }
                }

                if !self.is_v_wall(row, col) {
                    result_mask.extend(Self::mask_rectangle(cell_top, grid_left, cell_bottom, cell_left));
                } 

                if !self.is_v_wall(row, col + 1) {
                    result_mask.extend(Self::mask_rectangle(cell_top, cell_right, cell_bottom, grid_right))
                } else if self.cell_grid.contains_key(&(row, col)) && self.cell_grid.contains_key(&(row, col + 1)) {
                    if random::<f32>() < shortcut_probability {
                        result_mask.extend(Self::mask_rectangle(shortcut_top, cell_right, shortcut_bottom, cell_left + grid_spacing));
                    }
                }
            }
        }

        result_mask
    }

    pub fn write_maze(&self, out: impl Write) -> Result<(), io::Error> {
        let mut out = BufWriter::new(out);
        let f = self.pool.furthest_pair().unwrap();
        let start = self.pool.get(f.0).payload;
        let end = self.pool.get(f.1).payload;


        out.write_all(&(self.width as u32).to_be_bytes())?;
        out.write_all(&(self.height as u32).to_be_bytes())?;

        out.write_all(&(start.0 as u32).to_be_bytes())?;
        out.write_all(&(start.1 as u32).to_be_bytes())?;
        out.write_all(&(end.0 as u32).to_be_bytes())?;
        out.write_all(&(end.1 as u32).to_be_bytes())?;

        for row in 0..self.height {
            for col in 0..self.width {
                out.write(&[self.cell_to_byte(row, col)])?;
            }
        }

        Ok(())
    }

    fn north(b: u8) -> bool { (b & 0b1000) == 0b1000 }
    fn east(b: u8) -> bool { (b & 0b0100) == 0b0100 }
    fn west(b: u8) -> bool { (b & 0b0010) == 0b0010 }
    fn south(b: u8) -> bool { (b & 0b0001) == 0b0001 }

    pub fn validate_news_grid(news_grid: &HashMap<(usize, usize), u8>) -> Result<(),NewsGridError> {
        for (&(row, col), &b) in news_grid.iter() {
            // North checks
            if Self::north(b) {
                if row == 0 {
                    return Err(NewsGridError::ConnectedOutOfBounds {
                        cell: (row, col),
                        direction: Direction::North,
                    })
                }
                match news_grid.get(&(row - 1, col)) {
                    Some(&north_cell) => {
                        if !Self::south(north_cell) {
                            return Err(NewsGridError::UnrequitedConnection { 
                                linked: (row, col), 
                                unlinked: (row - 1, col), 
                                direction: Direction::North 
                            });
                        }
                    },
                    None => {
                        return Err(NewsGridError::ConnectedOutOfMask {
                            linked: (row, col),
                            missing: (row - 1, col),
                            direction: Direction::North,
                        });
                    },
                }
            }
            // East checks
            if Self::east(b) {
                match news_grid.get(&(row, col + 1)) {
                    Some(&east_cell) => {
                        if !Self::west(east_cell) {
                            return Err(NewsGridError::UnrequitedConnection {
                                linked: (row, col),
                                unlinked: (row, col + 1),
                                direction: Direction::East,
                            });
                        }
                    },
                    None => {
                        return Err(NewsGridError::ConnectedOutOfMask {
                            linked: (row, col),
                            missing: (row, col + 1),
                            direction: Direction::East,
                        });
                    },
                }
            }
            // West checks
            if Self::west(b) {
                if col == 0 {
                    return Err(NewsGridError::ConnectedOutOfBounds {
                        cell: (row, col),
                        direction: Direction::West,
                    });
                }
                match news_grid.get(&(row, col - 1)) {
                    Some(&west_cell) => {
                        if !Self::east(west_cell) {
                            return Err(NewsGridError::UnrequitedConnection {
                                linked: (row, col),
                                unlinked: (row, col - 1),
                                direction: Direction::West,
                            });
                        }
                    },
                    None => {
                        return Err(NewsGridError::ConnectedOutOfMask {
                            linked: (row, col),
                            missing: (row, col - 1),
                            direction: Direction::West,
                        })
                    },
                }
            }
            // South checks
            if Self::south(b) {
                match news_grid.get(&(row + 1, col)) {
                    Some(&south_cell) => {
                        if !Self::north(south_cell) {
                            return Err(NewsGridError::UnrequitedConnection { 
                                linked: (row, col), 
                                unlinked: (row + 1, col), 
                                direction: Direction::South
                            });
                        }
                    }
                    None => {
                        return Err(NewsGridError::ConnectedOutOfMask {
                            linked: (row, col),
                            missing: (row + 1, col),
                            direction: Direction::South,
                        });
                    },
                }
            }
        }

        return Ok(());
    }

    pub fn read_maze(input: impl Read) -> Result<Self, GridReadError> {
        let mut input = BufReader::new(input);

        let width = u32::from_be_bytes({
            let mut bytes = [0u8; 4];
            input.read_exact(&mut bytes)?;
            bytes
        }) as usize;
        let height = u32::from_be_bytes({
            let mut bytes = [0u8; 4];
            input.read_exact(&mut bytes)?;
            bytes
        }) as usize;

        let (_start_row, _start_col, _end_row, _end_col) = {
            let mut byte_bytes = [[0u8; 4]; 4];
            input.read_exact(&mut byte_bytes[0])?;
            input.read_exact(&mut byte_bytes[1])?;
            input.read_exact(&mut byte_bytes[2])?;
            input.read_exact(&mut byte_bytes[3])?;
            (
                u32::from_be_bytes(byte_bytes[0]) as usize,
                u32::from_be_bytes(byte_bytes[1]) as usize,
                u32::from_be_bytes(byte_bytes[2]) as usize,
                u32::from_be_bytes(byte_bytes[3]) as usize,
            )
        };
        
        let mut news_grid: HashMap<(usize, usize), u8> = HashMap::new();

        let mut node_bytes = input.bytes();

        for row in 0..height {
            for col in 0..width {
                match node_bytes.next() {
                    Some(b) => {
                        let b = b?;
                        // b == 0 -> this cell is not part of the maze, so we should skip this byte and move on to the next grid position
                        if b == 0 {
                            continue;
                        }
                        news_grid.insert((row, col), b);
                    },
                    None => {
                        return Err(GridReadError::NotEnoughBytes);
                    }
                }
            }
        }
        if let Some(_) = node_bytes.next() {
            return Err(GridReadError::TooManyBytes);
        }

        Self::validate_news_grid(&news_grid)?;
        // Here, news_grid is complete and ready to be used

        let mask: HashSet<(usize, usize)> = news_grid.keys().cloned().collect();

        let mut result = MaskedGrid::new(width, height, Box::new(move |row, col| {
            mask.contains(&(row, col))
        }));

        for (&(row, col), &b) in news_grid.iter() {
            if Self::north(b) {
                result.pool.link_cells(result.get_id_at(row, col).unwrap(), result.get_id_at(row - 1, col).unwrap(), true);
            }
            if Self::east(b) {
                result.pool.link_cells(result.get_id_at(row, col).unwrap(), result.get_id_at(row, col + 1).unwrap(), true);
            }
            if Self::west(b) {
                result.pool.link_cells(result.get_id_at(row, col).unwrap(), result.get_id_at(row, col - 1).unwrap(), true);
            }
            if Self::south(b) {
                result.pool.link_cells(result.get_id_at(row, col).unwrap(), result.get_id_at(row + 1, col).unwrap(), true);
            }
        }


        Ok(result)
    }
}
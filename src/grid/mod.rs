use std::collections::HashSet;

use rand::{
    random, rngs::ThreadRng, thread_rng, Rng,
};
use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

use crate::{
    dijkstra::{DijkstraPad, Distance},
    pool::{NodeId, Pool},
    sample_uniform,
};

pub mod walker;

pub struct FlatSquareCell {
    pub id: NodeId,
    row: usize,
    col: usize,
    north: Option<NodeId>,
    south: Option<NodeId>,
    east: Option<NodeId>,
    west: Option<NodeId>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

pub struct FlatSquareGrid {
    pub node_pool: Pool<FlatSquareCell>,
    node_grid: Vec<Vec<NodeId>>,
    pub width: usize,
    pub height: usize,
}

impl FlatSquareCell {
    pub fn new(id: NodeId, row: usize, col: usize) -> Self {
        FlatSquareCell {
            id,
            row,
            col,
            north: None,
            south: None,
            east: None,
            west: None,
        }
    }

    pub fn at_direction(&self, direction: Direction) -> Option<NodeId> {
        match direction {
            Direction::North => self.north,
            Direction::South => self.south,
            Direction::East => self.east,
            Direction::West => self.west,
        }
    }

    pub fn new_constructor(row: usize, col: usize) -> impl FnOnce(NodeId) -> Self {
        move |id| FlatSquareCell::new(id, row, col)
    }

    pub fn neighbors(&self) -> Vec<NodeId> {
        let mut r = vec![];
        r.extend(self.north);
        r.extend(self.south);
        r.extend(self.east);
        r.extend(self.west);
        r
    }
}

pub enum BinaryTreeProbabilityType {
    Constant(f64),
    PerCell(Box<dyn Fn(usize, usize) -> f64>),
}
pub struct BinaryTreeSettings {
    probability_north: BinaryTreeProbabilityType,
}

impl BinaryTreeSettings {
    pub fn with_probability_north(p: f64) -> Self {
        BinaryTreeSettings {
            probability_north: BinaryTreeProbabilityType::Constant(p),
        }
    }

    pub fn with_custom_probability(f: impl Fn(usize, usize) -> f64 + 'static) -> Self {
        BinaryTreeSettings {
            probability_north: BinaryTreeProbabilityType::PerCell(Box::new(f)),
        }
    }

    pub fn get_probability(&self, row: usize, col: usize) -> f64 {
        use BinaryTreeProbabilityType as BT;
        match &self.probability_north {
            BT::Constant(p) => *p,
            BT::PerCell(f) => f(row, col),
        }
    }
}

impl FlatSquareGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut node_pool: Pool<FlatSquareCell> = Pool::new();
        let mut node_grid = vec![vec![]; height];
        for row in 0..height {
            for col in 0..width {
                let id = node_pool.new_node(FlatSquareCell::new_constructor(row, col));
                node_grid[row].push(id);
            }
        }
        let mut g = FlatSquareGrid {
            node_pool,
            node_grid,
            width,
            height,
        };
        g.stitch();
        g
    }

    pub fn size(&self) -> usize {
        self.width * self.height
    }

    fn stitch(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let cell_id = self.node_grid[row][col];
                let cell = &mut self.node_pool.get_mut(cell_id).payload;
                if row > 0 {
                    cell.north = Some(self.node_grid[row - 1][col]);
                }
                if row < self.height - 1 {
                    cell.south = Some(self.node_grid[row + 1][col]);
                }
                if col > 0 {
                    cell.west = Some(self.node_grid[row][col - 1]);
                }
                if col < self.width - 1 {
                    cell.east = Some(self.node_grid[row][col + 1]);
                }

                if row > 0 {
                    self.node_pool
                        .make_adjacent(cell_id, self.node_grid[row - 1][col], true);
                }
                if row < self.height - 1 {
                    self.node_pool
                        .make_adjacent(cell_id, self.node_grid[row + 1][col], true);
                }
                if col > 0 {
                    self.node_pool
                        .make_adjacent(cell_id, self.node_grid[row][col - 1], true);
                }
                if col < self.width - 1 {
                    self.node_pool
                        .make_adjacent(cell_id, self.node_grid[row][col + 1], true);
                }
            }
        }
    }

    /*pub fn generate_shuffle(&self, rng: &mut ThreadRng) -> Vec<NodeId> {
        let mut ids = (0..self.node_pool.nodes.len()).collect::<Vec<NodeId>>();
        ids.shuffle(rng);
        ids
    }*/

    pub fn get_position_by_id(&self, id: NodeId) -> (usize, usize) {
        let cell = self.get_by_id(id);
        (cell.row, cell.col)
    }

    pub fn get_by_id(&self, id: NodeId) -> &FlatSquareCell {
        &self.node_pool.get(id).payload
    }

    pub fn get_by_id_mut(&mut self, id: NodeId) -> &mut FlatSquareCell {
        &mut self.node_pool.get_mut(id).payload
    }

    pub fn get_by_position(&self, row: usize, col: usize) -> &FlatSquareCell {
        if row >= self.height || col >= self.width {
            panic!("Access out of bounds!");
        }
        self.get_by_id(self.node_grid[row][col])
    }

    pub fn get_by_position_mut(&mut self, row: usize, col: usize) -> &mut FlatSquareCell {
        self.get_by_id_mut(self.node_grid[row][col])
    }

    pub fn is_linked(&self, here: NodeId, there: NodeId) -> bool {
        self.node_pool.is_linked(here, there)
    }

    pub fn is_linked_at(&self, row1: usize, col1: usize, row2: usize, col2: usize) -> bool {
        self.is_linked(
            self.get_by_position(row1, col1).id,
            self.get_by_position(row2, col2).id,
        )
    }

    pub fn link_cells_at(&mut self, here: (usize, usize), there: (usize, usize)) {
        assert!(here.0 < self.width && there.0 < self.width);
        assert!(here.1 < self.height && there.1 < self.height);
        let cell1 = self.get_by_position(here.0, here.1).id;
        let cell2 = self.get_by_position(there.0, there.1).id;
        self.node_pool.link_cells(cell1, cell2, true);
    }

    pub fn binary_tree(&mut self, settings: BinaryTreeSettings) {
        for cell_id in self.node_pool.iter_node_ids() {
            let (row, col) = self.get_position_by_id(cell_id);
            let mut nebs = vec![];
            nebs.extend(self.get_by_id(cell_id).north);
            nebs.extend(self.get_by_id(cell_id).east);
            if nebs.len() == 1 {
                self.node_pool.link_cells(cell_id, nebs[0], true);
            } else if nebs.len() == 2 {
                self.node_pool.link_cells(
                    cell_id,
                    nebs[if random::<f64>() < settings.get_probability(row, col) {
                        0
                    } else {
                        1
                    }],
                    true,
                );
            }
        }
    }

    pub fn sidewinder(&mut self) {
        let mut rng = thread_rng();
        for row in 1..self.height {
            let mut hallway_start = 0;
            while hallway_start < self.width - 1 {
                let taken = Self::take_out_of(self.width - hallway_start, &mut rng);

                for dcol in 0..taken - 1 {
                    self.link_cells_at(
                        (row, hallway_start + dcol),
                        (row, hallway_start + dcol + 1),
                    );
                }
                let hallway_door: usize = rng.gen_range(0..taken);
                if row != 0 {
                    self.link_cells_at(
                        (row, hallway_start + hallway_door),
                        (row - 1, hallway_start + hallway_door),
                    );
                }
                hallway_start += taken;
            }
            if hallway_start == self.width - 1 {
                self.link_cells_at((row, self.width - 1), (row - 1, self.width - 1));
            }
        }
        for col in 0..self.width - 1 {
            self.link_cells_at((0, col), (0, col + 1));
        }
    }

    pub fn aldous_broder(&mut self, rng: &mut ThreadRng) {
        // Pick a random starting cell
        let mut cell = self.node_pool.get_random_node_id(rng);
        let mut unvisited = self.size() - 1;
        while unvisited > 0 {
            let neighbors = self.get_by_id(cell).neighbors();
            let neighbor = neighbors[rng.gen_range(0..neighbors.len())];
            if self.node_pool.get(neighbor).links.is_empty() {
                self.node_pool.link_cells(cell, neighbor, true);
                unvisited -= 1;
            }
            cell = neighbor;
        }
    }

    pub fn hunt_and_kill(&mut self, rng: &mut ThreadRng) {
        self.node_pool.hunt_and_kill(rng);
        return;
    }

    pub fn recursive_backtracker(&mut self, rng: &mut ThreadRng) {
        let (mut visited, mut stack) = {
            let start = self.node_pool.get_arbitrary_node_id();
            (HashSet::from([start]), vec![start])
        };
        while let Some(&top_of_stack) = stack.last() {
            let viable_cells: Vec<NodeId> = self
                .node_pool
                .unvisited_neighborhood_of(&visited, top_of_stack)
                .into_iter()
                .collect();
            if viable_cells.is_empty() {
                stack.pop();
                continue;
            } else {
                let next_cell = *sample_uniform(&viable_cells, rng);
                self.node_pool.link_cells(top_of_stack, next_cell, true);
                stack.push(next_cell);
                visited.insert(next_cell);
            }
        }
    }

    fn take_out_of(max: usize, rng: &mut ThreadRng) -> usize {
        assert_ne!(max, 0);
        let mut taken = 1;
        while rng.gen() && taken < max {
            taken += 1;
        }
        taken
    }

    pub fn text_print(&self) -> String {
        let mut result = String::new();
        result.push_str("+");
        for _ in 0..self.width {
            result.push_str("---+");
        }
        result.push_str("\n");

        for row in 0..self.height {
            let mut row_line = String::new();
            row_line.push('|');

            for col in 0..self.width {
                row_line.push_str("   ");
                if let Some(other_cell_id) = self.get_by_position(row, col).east {
                    let east_cell = self.get_by_id(other_cell_id);
                    if self.is_linked_at(row, col, east_cell.row, east_cell.col) {
                        row_line.push(' ');
                    } else {
                        row_line.push('|');
                    }
                } else {
                    row_line.push('|');
                }
            }
            row_line.push('\n');
            row_line.push('+');
            for col in 0..self.width {
                if let Some(other_cell_id) = self.get_by_position(row, col).south {
                    let south_cell = self.get_by_id(other_cell_id);
                    if self.is_linked_at(row, col, south_cell.row, south_cell.col) {
                        row_line.push_str("   ");
                    } else {
                        row_line.push_str("---");
                    }
                } else {
                    row_line.push_str("---");
                }
                row_line.push('+');
            }
            row_line.push('\n');
            result.push_str(&row_line);
        }
        result
    }

    pub fn image_print(
        &self,
        cell_size: usize,
        padding: usize,
        paint_function: impl Fn(NodeId) -> Paint<'static>,
    ) -> Pixmap {
        let image_width = self.width * cell_size + 2 * padding;
        let image_height = self.height * cell_size + 2 * padding;
        let mut pixmap = Pixmap::new(image_width as u32, image_height as u32).unwrap();

        let paint = {
            let mut paint = Paint::default();
            paint.set_color_rgba8(0, 0, 0, 255);
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

        let path = {
            let mut pb = PathBuilder::new();

            // First do the south and west walls of each cell (when applicable)
            // Do the south and west walls of each cell in large region
            for row in 1..self.height {
                for col in 1..self.width {
                    let top = (row * cell_size + padding) as f32;
                    let bottom = ((row + 1) * cell_size + padding) as f32;
                    let left = (col * cell_size + padding) as f32;
                    let right = ((col + 1) * cell_size + padding) as f32;
                    if !self.is_linked_at(row, col, row - 1, col) {
                        pb.move_to(left, top);
                        pb.line_to(right, top);
                    }
                    if !self.is_linked_at(row, col, row, col - 1) {
                        pb.move_to(left, top);
                        pb.line_to(left, bottom);
                    }
                }
            }

            // Do the west walls of the bottom strip
            for col in 1..self.width {
                let top = padding as f32;
                let bottom = (cell_size + padding) as f32;
                let left = (col * cell_size + padding) as f32;
                let _right = ((col + 1) * cell_size + padding) as f32;
                if !self.is_linked_at(0, col, 0, col - 1) {
                    pb.move_to(left, top);
                    pb.line_to(left, bottom);
                }
            }

            // Do the south walls of the left strip
            for row in 1..self.height {
                let top = (row * cell_size + padding) as f32;
                let _bottom = ((row + 1) * cell_size + padding) as f32;
                let left = padding as f32;
                let right = (cell_size + padding) as f32;
                if !self.is_linked_at(row, 0, row - 1, 0) {
                    pb.move_to(left, top);
                    pb.line_to(right, top);
                }
            }
            // Then finish off by drawing the enclosing rectangle of entire maze
            pb.move_to(padding as f32, padding as f32);
            pb.push_rect(
                Rect::from_ltrb(
                    padding as f32,
                    padding as f32,
                    (padding + cell_size * self.width) as f32,
                    (padding + cell_size * self.height) as f32,
                )
                .unwrap(),
            );

            pb.finish().unwrap()
        };

        // Paint the interior of every cell according to the `paint_function`
        for row in 0..self.height {
            for col in 0..self.width {
                let top = (row * cell_size + padding) as f32;
                let bottom = ((row + 1) * cell_size + padding) as f32;
                let left = (col * cell_size + padding) as f32;
                let right = ((col + 1) * cell_size + padding) as f32;

                pixmap.fill_rect(
                    Rect::from_ltrb(left, top, right, bottom).unwrap(),
                    &paint_function(self.get_by_position(row, col).id),
                    Transform::identity(),
                    None,
                );
            }
        }

        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);

        return pixmap;
    }

    pub fn image_print_distances(
        &self,
        cell_size: usize,
        padding: usize,
        start_node: NodeId,
        color_function: impl Fn(f64) -> Color,
    ) -> Pixmap {
        let distances = DijkstraPad::new(&self.node_pool, start_node).perform();
        let max_finite_distance = distances
            .pool
            .payloads()
            .map(|d| match d {
                Distance::Infinite => 0,
                Distance::Finite(dist) => *dist,
            })
            .max()
            .unwrap_or(0) as f64;

        if max_finite_distance == 0.0 {
            self.image_print(cell_size, padding, |_| {
                let mut p = Paint::default();
                p.set_color_rgba8(u8::MAX, u8::MAX, u8::MAX, u8::MAX);
                p
            })
        } else {
            self.image_print(cell_size, padding, |node_id| {
                let dist = distances.pool.get(node_id).payload.as_finite().unwrap_or(0) as f64;
                let normalized_distance = dist / max_finite_distance;
                let mut p = Paint::default();
                p.set_color(color_function(normalized_distance));
                p
            })
        }
    }
}

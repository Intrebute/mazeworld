use std::collections::HashSet;

use rand::{rngs::ThreadRng, seq::IteratorRandom};

use crate::{pool::{NodeId, Pool}, sample_uniform};

use super::{FlatSquareGrid, Direction};

#[derive(PartialEq, Eq)]
pub struct Walker {
    pub start_node: NodeId,
    pub path: Vec<NodeId>,
}

enum RepeatedDirection {
    Start,
    Middle(usize),
    Never,
}

impl Walker {
    pub fn new(start_node: NodeId) -> Self {
        Walker {
            start_node,
            path: vec![],
        }
    }

    pub fn random_loop_erased_step<T>(&mut self, pool: &Pool<T>, rng: &mut ThreadRng) {
        let new_head = pool.neighborhood_of(self.final_node()).into_iter().choose(rng);
        match new_head {
            Some(new_head) => self.loop_erased_step(new_head),
            None => panic!("Attempted to walk out of node {} with empty neighborhood.", self.final_node()),
        }
    }

    /// Appends a new step in the specified direction.
    pub fn loop_erased_step(&mut self, next_cell: NodeId) {
        match self.steps_on(next_cell) {
            RepeatedDirection::Start => {
                self.path.truncate(0);
            },
            RepeatedDirection::Middle(i) => self.path.truncate(i),
            RepeatedDirection::Never => self.path.push(next_cell),
        }
    }

    /// Checks whether the path already steps on the given node. If so, returns the index of the step that lands on the repeated square.
    fn steps_on(&self, node: NodeId) -> RepeatedDirection {
        if self.start_node == node {
            return RepeatedDirection::Start;
        }
        for i in 0..self.path.len() {
            if self.path[i] == node {
                return RepeatedDirection::Middle(i);
            }
        }
        return RepeatedDirection::Never;
    }

    fn next_step_at_direction(&self, grid: &FlatSquareGrid, direction: Direction) -> Option<NodeId> {
        grid.get_by_id(self.final_node()).at_direction(direction)
    }

    pub fn total_path(&self) -> Vec<NodeId> {
        let mut v = self.path.clone();
        v.insert(0,self.start_node);
        v
    }

    pub fn final_node(&self) -> NodeId {
        *self.path.last().unwrap_or(&self.start_node)
    }

    pub fn loop_erased_walk_into_haystack<N>(&mut self, pool: &Pool<N>, targets: &HashSet<NodeId>, rng: &mut ThreadRng) {
        while !targets.contains(&self.final_node()) {
            self.random_loop_erased_step(pool, rng)
        }
    }

    pub fn carve_path<T>(&self, pool: &mut Pool<T>) {
        let path = self.total_path();
        for pair in path.windows(2) {
            pool.link_cells(pair[0], pair[1], true);
        }
    }
}

impl FlatSquareGrid {

    /// Wilson's algorithm.
    /// 
    /// ~~Bad.~~ Fixed! Good!
    pub fn wilson(&mut self, rng: &mut ThreadRng) {
        let mut starts_list = self.node_pool.iter_node_ids().collect::<Vec<NodeId>>();
        let mut visited_set: HashSet<NodeId> = HashSet::new();
        if let Some(needle) = starts_list.pop() {
            visited_set.insert(needle);   
        }
        while let Some(start) = starts_list.pop() {
            let path = {
                let mut path = Walker::new(start);
                path.loop_erased_walk_into_haystack(&self.node_pool, &visited_set, rng);
                path
            };
            let path_nodes = path.total_path();
            path.carve_path(&mut self.node_pool);
            visited_set.extend(path_nodes.into_iter());
        }
    }

    
}

#[cfg(test)]
mod tests {
    use crate::grid::Direction;

    use super::*;

    #[test]
    fn non_overlapping_path() {
        let grid = FlatSquareGrid::new(4,4);
        let mut w = Walker::new(grid.node_pool.nodes[0].id);
        assert_eq!(w.total_path(), vec![0]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::South).unwrap());
        assert_eq!(w.total_path(), vec![0,4]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::East).unwrap());
        assert_eq!(w.total_path(), vec![0,4,5]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::North).unwrap());
        assert_eq!(w.total_path(), vec![0,4,5,1]);
    }

    #[test]
    fn overlapping_middle_path() {
        let grid = FlatSquareGrid::new(4,4);
        let mut w = Walker::new(grid.node_pool.nodes[5].id);
        assert_eq!(w.total_path(), vec![5]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::South).unwrap());
        assert_eq!(w.total_path(), vec![5,9]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::East).unwrap());
        assert_eq!(w.total_path(), vec![5,9,10]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::East).unwrap());
        assert_eq!(w.total_path(), vec![5,9,10,11]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::North).unwrap());
        assert_eq!(w.total_path(), vec![5,9,10,11,7]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::West).unwrap());
        assert_eq!(w.total_path(), vec![5,9,10,11,7,6]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::South).unwrap());
        assert_eq!(w.total_path(), vec![5,9]);
    }

    #[test]
    fn overlapping_start_path() {
        let grid = FlatSquareGrid::new(4,4);
        let mut w = Walker::new(grid.node_pool.nodes[5].id);
        assert_eq!(w.total_path(), vec![5]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::South).unwrap());
        assert_eq!(w.total_path(), vec![5,9]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::East).unwrap());
        assert_eq!(w.total_path(), vec![5,9,10]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::East).unwrap());
        assert_eq!(w.total_path(), vec![5,9,10,11]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::North).unwrap());
        assert_eq!(w.total_path(), vec![5,9,10,11,7]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::West).unwrap());
        assert_eq!(w.total_path(), vec![5,9,10,11,7,6]);
        w.loop_erased_step(w.next_step_at_direction(&grid, Direction::West).unwrap());
        assert_eq!(w.total_path(), vec![5]);
    }
}
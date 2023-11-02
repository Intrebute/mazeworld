use std::collections::HashMap;

use crate::pool::{NodeId, Pool};


pub struct TriangleCell {
    pub id: NodeId,
    pub row: usize,
    pub col: usize,
}

pub struct TriangleGrid {
    pub cell_coordinates: HashMap<(usize, usize),NodeId>,
    pub pool: Pool<()>,
    pub width: usize,
    pub height: usize,
}

/*impl TriangleGrid {
    pub fn adjacent_triangles(&self, row: usize, col: usize) -> impl Iterator<Item = NodeId> {
        let mut ns = vec![];
        ns.extend(col.checked_sub(1));
        ns.extend(if col == self.width - 1 { None } else { Some(col + 1) });
        if self.points_up(row, col) {
            ns.extend(if row == self.height - 1 { None } else { Some(row + 1) });
        } else {
            ns.extend(if row == 0 { None } else { Some(row - 1) });
        }
        ns.into_iter()
    }

    fn points_up(&self, row: usize, col: usize) -> bool {
        (row + col) % 2 == 0
    }
}*/
use std::{collections::HashSet, fmt::Display};

use crate::pool::{Pool, NodeId};



#[derive(Clone, Copy, Debug)]
pub enum Distance {
    Finite(usize),
    Infinite,
}

pub struct DijkstraPad {
    pub pool: Pool<Option<Distance>>,
    pub start_node: NodeId,
}

#[derive(Debug)]
pub struct Distances {
    pub pool: Pool<Distance>,
    pub start_node: NodeId,
}

impl Distance {
    pub fn finite(d: usize) -> Self {
        Self::Finite(d)
    }

    pub fn infinite() -> Self {
        Self::Infinite
    }

    pub fn as_finite(self) -> Option<usize> {
        match self {
            Self::Infinite => None,
            Self::Finite(d) => Some(d)
        }
    }
}

impl std::ops::Add<usize> for Distance {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        use Distance as D;
        match self {
            D::Finite(d) => D::Finite(d + rhs),
            D::Infinite => D::Infinite
        }
    }
}

impl Display for Distance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Distance as D;
        match self {
            D::Finite(d) => write!(f, "{}", d),
            D::Infinite => write!(f,"∞"),
        }
    }
}

impl DijkstraPad {
    pub fn new<T>(source: &Pool<T>, start_node: NodeId) -> Self {
        use Distance as D;
        let pool = source.map_nodes(|n| if n.id == start_node { Some(D::finite(0)) } else { None });
        DijkstraPad {
            pool, start_node
        }
    }

    pub fn perform(mut self) -> Distances {
        let mut frontier: HashSet<NodeId> = HashSet::new();
        frontier.insert(self.start_node);
        while !frontier.is_empty() {
            let mut new_frontier: HashSet<NodeId> = HashSet::new();
            for cell in &frontier {
                let curr_distance = self.pool.get(*cell).payload.unwrap();
                let neighbors: HashSet<_> = self.pool.passages_of(*cell).into_iter().filter(|&c| self.pool.get(c).payload.is_none()).collect();
                for neighbor in neighbors {
                    self.pool.get_mut(neighbor).payload = Some(curr_distance + 1);
                    new_frontier.insert(neighbor);
                }
            }
            frontier = new_frontier;
        }
        let new_pool = self.pool.map_nodes(|n| n.payload.unwrap_or(Distance::Infinite));
        Distances {
            pool: new_pool,
            start_node: self.start_node,
        }
    }
}
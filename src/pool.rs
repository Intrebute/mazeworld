use std::{collections::HashSet, ops::{Index, IndexMut}, fmt::Display};

use partitions::{PartitionVec, partition_vec};
use rand::{rngs::ThreadRng, distributions::Uniform, prelude::Distribution, Rng};

use crate::{sample_uniform, dijkstra::DijkstraPad};


#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct NodeId(usize);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

impl PartialEq<usize> for NodeId {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Node<T> {
    pub id: NodeId,
    pub links: HashSet<NodeId>,
    pub adjacencies: HashSet<NodeId>,
    pub payload: T,
}

/// A pool of nodes and connections.
/// 
/// A pool consists abstractly of two graphs, the link graph and the adjacency graph.
/// 
/// The adjacency graph specifies what nodes _might_ be connected. It defines the geometry and layout of the graph, in a way.
/// 
/// The link graph is a subgraph of the adjacency graph. Two nodes can be linked only if they are adjacent. This allows algorithms to 
/// poll from the neighborhood of a node to then choose which ones to turn into links, which can then be thought of as passages.
#[derive(Debug, PartialEq, Eq)]
pub struct Pool<T> {
    pub nodes: Vec<Node<T>>,
}

impl<T> Display for Pool<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for n in &self.nodes {
            write!(f, "{} -> ", n.id)?;
            for neigh in self.neighborhood_of(n.id) {
                if self.is_linked(n.id, neigh) {
                    write!(f, "[{}]", neigh.0)?;
                } else {
                    write!(f, "{} ", neigh.0)?;
                }
            }
            writeln!(f)?
        }
        Ok(())
    }
}

/// Represents the result of searching the frontier of a set of visited nodes. Used by [`Pool::scan_frontier`].
pub enum FrontierSearchResult {
    /// A node outside the visited set, adjacent to one that is, has been found.
    Found {
        /// The node outside the visited set.
        unvisited: NodeId,
        /// The node in the visited set adjacent to the unvisited one.
        visited: NodeId,
    },
    /// There were no unvisited cells adjacent to any in the visited set.
    NoFrontier,
}

impl<T> Node<T> {
    pub fn new(id: NodeId, construct: impl FnOnce(NodeId) -> T) -> Self {
        Node {
            id,
            links: HashSet::new(),
            adjacencies: HashSet::new(),
            payload: construct(id),
        }
    }
}

impl<T> Index<NodeId> for Pool<T> {
    type Output = Node<T>;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl<T> IndexMut<NodeId> for Pool<T> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        Pool { nodes: vec![] }
    }

    pub fn iter_node_ids(&self) -> impl Iterator<Item = NodeId> {
        (0..self.nodes.len()).map(|i| NodeId(i))
    }

    /// Returns a node selected uniformly over all nodes in the pool.
    pub fn get_random_node_id(&self, rng: &mut ThreadRng) -> NodeId {
        sample_uniform(&self.nodes, rng).id
    }

    /// Returns an unspecified but valid node in the pool. It might always be the same node.
    pub fn get_arbitrary_node_id(&self) -> NodeId {
        self.nodes[0].id
    }

    /// Constructs a new node and returns its id.
    /// 
    /// The `payload` of the new node is constructed by the `construct` function, which is given access to the new node id.
    pub fn new_node(&mut self, construct: impl FnOnce(NodeId) -> T) -> NodeId {
        let new_id = NodeId(self.nodes.len());
        self.nodes.push(Node::new(new_id, construct));
        new_id
    }

    pub fn aldous_broder(&mut self, rng: &mut ThreadRng) {
        let mut cell = self.get_random_node_id(rng);
        let mut unvisited_count = self.nodes.len() - 1;

        while unvisited_count > 0 {
            let neighbors: Vec<NodeId> = self.neighborhood_of(cell).into_iter().collect();
            let random_neighbor = neighbors[rng.gen_range(0..neighbors.len())];
            if self.get(random_neighbor).links.is_empty() {
                self.link_cells(cell, random_neighbor, true);
                unvisited_count -= 1;
            }
            cell = random_neighbor;
        }
    }

    pub fn hunt_and_kill(&mut self, rng: &mut ThreadRng) {
        let mut visited: HashSet<NodeId> = HashSet::new();
        if let Some(first) = self.nodes.first() {
            // If there are any nodes at all, start off with the first one
            visited.insert(first.id);
        } else {
            // Otherwise, we're done and we leave
            return;
        }

        while let FrontierSearchResult::Found{ unvisited: mut current_cell, visited: visited_root}
        = self.scan_frontier(&visited) {
            self.link_cells(current_cell, visited_root, true);
            visited.insert(current_cell);
            let mut walls: Vec<NodeId> = self.walls_of(current_cell).into_iter().filter(|n| {
                !visited.contains(n)
            }).collect();

            while !walls.is_empty() {
                let next_cell = *sample_uniform(&walls, rng);
                self.link_cells(current_cell, next_cell, true);
                current_cell = next_cell;
                visited.insert(current_cell);
                walls = self.walls_of(current_cell).into_iter().filter(|n| !visited.contains(n)).collect();
            }
        }
    }

    /// Finds a node in the pool adjacent to nodes in the `visited` set. The node itself will not be in `visited`.
    pub fn scan_frontier(&self, visited: &HashSet<NodeId>) -> FrontierSearchResult {
        for node in self.nodes.iter().filter(|n| {
            !visited.contains(&n.id)
        }) {
            for wall in self.walls_of(node.id) {
                if visited.contains(&wall) {
                    return FrontierSearchResult::Found { unvisited: node.id, visited: wall };
                }
            }
        }

        return FrontierSearchResult::NoFrontier;
    }

    pub fn furthest_pair(&self) -> Option<(NodeId, NodeId)> {
        let arbitrary_id = self.get_arbitrary_node_id();
        let distances_from_arbitrary = DijkstraPad::new(self, arbitrary_id).perform();
        let furthest_from_arbitrary = distances_from_arbitrary.pool.nodes.into_iter().max_by_key(|n| {
            n.payload.as_finite().unwrap_or(0)
        })?.id;
        let distances_from_furthest = DijkstraPad::new(self, furthest_from_arbitrary).perform();
        let furthest_from_furthest = distances_from_furthest.pool.nodes.into_iter().max_by_key(|n| {
            n.payload.as_finite().unwrap_or(0)
        })?.id;

        Some((furthest_from_arbitrary, furthest_from_furthest))
    }

    /// Connects all nodes according to all adjacencies present.
    pub fn debug_connect_all(&mut self) where T: Clone {
        for n in self.nodes.clone() {
            for neigh in self.neighborhood_of(n.id) {
                self.link_cells(n.id, neigh, true);
            }
        }
    }

    /// Checks if the adjacency graph is connected.
    pub fn is_adjacently_connected(&self) -> bool {
        let mut nodes_partitions: PartitionVec<()> = partition_vec![(); self.nodes.len()];
        let node_count = self.nodes.len();
        for (i, node) in self.nodes.iter().enumerate() {
            println!("Visited {}/{} ({} %)", i, node_count, (i as f64 / node_count as f64) * 100.0);
            for neighbor in node.adjacencies.iter() {
                nodes_partitions.union(node.id.0, neighbor.0);
            }
        }
        nodes_partitions.amount_of_sets() == 1
        /*
        println!("Checking adjacently connected...");
        if let Some(start) = self.nodes.first() {
            let ids = self.nodes.iter().map(|n| n.id).collect::<HashSet<NodeId>>();
            let mut frontier: Vec<NodeId> = Vec::new();
            let mut visited: HashSet<NodeId> = HashSet::new();
            frontier.push(start.id);
            while let Some(node) = frontier.pop() {
                {
                    let vl = visited.len();
                    let tl = ids.len();
                    println!("Visited {}/{} ({}%)", vl, tl, vl as f64 / tl as f64 * 100.0);
                }
                visited.insert(node);
                let new_chip = &(&self.neighborhood_of(node) - &visited) - &frontier.iter().cloned().collect::<HashSet<_>>();
                frontier.extend(new_chip);
            }

            return ids == visited;
        } else {
            true
        }*/
    }

    /// Links two adjacent nodes.
    /// 
    /// # Panics
    /// 
    /// Panics if the two cells are not marked as adjacent to each other.
    pub fn link_cells(&mut self, here: NodeId, there: NodeId, bidirectional: bool) {
        assert!(self[here].adjacencies.contains(&there), "Attempted to link non-adjacent cells {} and {}", here, there);
        self[here].links.insert(there);
        if bidirectional {
            assert!(self[there].adjacencies.contains(&here), "Attempted to link non-adjacent cells {} and {}", there, here);
            self[there].links.insert(here);
        }
    }

    /// Marks two nodes as adjacent. Only adjacent nodes can be then linked.
    pub fn make_adjacent(&mut self, here: NodeId, there: NodeId, bidirectional: bool) {
        self[here].adjacencies.insert(there);
        if bidirectional {
            self[there].adjacencies.insert(here);
        }
    }

    pub fn unlink_cells(&mut self, here: NodeId, there: NodeId, bidirectional: bool) {
        self[here].links.remove(&there);
        if bidirectional {
            self[there].links.remove(&here);
        }
    }

    /// Computes the set of nodes adjacent to node `id`. Note that this is not the same as the passages into and out of a cell.
    /// For linked passages, use `passages_of`
    pub fn neighborhood_of(&self, id: NodeId) -> HashSet<NodeId> {
        self[id].adjacencies.clone()
    }

    pub fn get(&self, id: NodeId) -> &Node<T> {
        &self[id]
    }

    pub fn get_mut(&mut self, id: NodeId) -> &mut Node<T> {
        &mut self[id]
    }

    /// Checks if the cells `here` and `there` are connected by a passage.
    pub fn is_linked(&self, here: NodeId, there: NodeId) -> bool {
        self[here].links.contains(&there)
    }

    pub fn map_nodes<U>(&self, f: impl Fn(&Node<T>) -> U) -> Pool<U> {
        let new_nodes: Vec<Node<U>> = self.nodes.iter().map(|n| {
            let mut nn = Node::new(n.id, |_| f(n));
            nn.links = n.links.clone();
            nn.adjacencies = n.adjacencies.clone();
            nn
        }).collect();
        Pool {
            nodes: new_nodes,
        }
    }

    pub fn unvisited_neighborhood_of(&self, visited: &HashSet<NodeId>, id: NodeId) -> HashSet<NodeId> {
        let ns = self.neighborhood_of(id);
        ns.difference(visited).cloned().collect()
    }

    /// Computes the set of cells accessible by passages from `id`. Note that this is not the same as the cells adjacent to it.
    /// For adjacent cells, use `neighborhood_of`
    pub fn passages_of(&self, id: NodeId) -> HashSet<NodeId> {
        self.get(id).links.clone()
    }

    pub fn walls_of(&self, id: NodeId) -> HashSet<NodeId> {
        self.neighborhood_of(id).difference(&self.passages_of(id)).cloned().collect()
    }

    pub fn payloads(&self) -> impl Iterator<Item = &T> {
        self.nodes.iter().map(|n| &n.payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let mut pool: Pool<()> = Pool::new();
        let node1 = pool.new_node(|_| ());
        let node2 = pool.new_node(|_| ());
        let node3 = pool.new_node(|_| ());

        assert!(!pool.is_adjacently_connected());

        pool.make_adjacent(node1, node2, true);

        assert!(!pool.is_adjacently_connected());

        pool.make_adjacent(node1, node3, true);
        
        assert!(pool.is_adjacently_connected());
    }
}
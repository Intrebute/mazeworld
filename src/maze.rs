use std::io::{self, Write};

use rand::rngs::ThreadRng;
use tiny_skia::{Pixmap, Paint};

use crate::{masked_grid::MaskedGrid, pool::NodeId, polar_grid::PolarGrid, lerp::multi_lerp, color_gradients};



pub enum Maze {
    MaskedMaze {
        maze: MaskedGrid,
        start: NodeId,
        end: NodeId,
    },
    RadialMaze {
        maze: PolarGrid,
        start: NodeId,
        end: NodeId,
    }
}

pub enum Algorithm {
    AldousBroder,
    HuntAndKill,
}

impl Maze {
    pub fn new_unmasked_cartesian(width: usize, height: usize, algo: Algorithm, rng: &mut ThreadRng) -> Self {
        let mut g = MaskedGrid::new_unmasked(width, height);
        match algo {
            Algorithm::AldousBroder => { g.aldous_broder(rng); },
            Algorithm::HuntAndKill => { g.hunt_and_kill(rng); },
        }
        let furthest_pair = g.pool.furthest_pair().unwrap();
        Self::MaskedMaze { maze: g, start: furthest_pair.0, end: furthest_pair.1 }
    }

    pub fn new_masked_cartesian(width: usize, height: usize, mask: Box<dyn Fn(usize, usize) -> bool>, algo: Algorithm, rng: &mut ThreadRng) -> Self {
        let mut g = MaskedGrid::new(width, height, mask);
        match algo {
            Algorithm::AldousBroder => { g.aldous_broder(rng); },
            Algorithm::HuntAndKill => { g.hunt_and_kill(rng); },
        }
        let furthest_pair = g.pool.furthest_pair().unwrap();
        Self::MaskedMaze { maze: g, start: furthest_pair.0, end: furthest_pair.1 }
    }

    pub fn new_unmasked_radial(starting_branch_count: usize, ring_count: usize, algo: Algorithm, rng: &mut ThreadRng) -> Self {
        let mut g = PolarGrid::new(starting_branch_count, ring_count);
        //g.pool.debug_connect_all();
        match algo {
            Algorithm::AldousBroder => { g.pool.aldous_broder(rng); },
            Algorithm::HuntAndKill => { g.pool.hunt_and_kill(rng); },
        }
        let furthest_pair = g.pool.furthest_pair().unwrap();
        Self::RadialMaze { maze: g, start: furthest_pair.0, end: furthest_pair.1 }
    }

    pub fn print_image(&self, width: usize, padding: usize) -> Pixmap {
        match self {
            Maze::MaskedMaze { maze, start, end } => {
                let cell_size = (width - 2 * padding) / maze.width;
                let mouse_icon = Pixmap::load_png("mouse.png").unwrap();
                let cheese_icon = Pixmap::load_png("cheese.png").unwrap();
                let pix = maze.print_image(cell_size, padding, true, |n| {
                    let mut paint = Paint::default();
                    paint.set_color_rgba8(u8::MAX, u8::MAX, u8::MAX, u8::MAX);
                    /*if n == *start {
                        paint.set_color_rgba8(0, 38, u8::MAX, u8::MAX);
                    } else if n == *end {
                        paint.set_color_rgba8(u8::MAX, 106, 0, u8::MAX);
                    } else {
                        paint.set_color_rgba8(u8::MAX, u8::MAX, u8::MAX, u8::MAX);
                    }*/
                    paint
                }, vec![(*start, mouse_icon), (*end, cheese_icon)]);

                pix
            },
            Maze::RadialMaze { maze, start, end } => {
                let radius = (width - 2 * padding) / 2;
                maze.print_image_distances(radius, padding, *start,
                    multi_lerp(color_gradients::fire_colors())
                )
            },
        }
    }

    pub fn write_maze(&self, out: impl Write) -> Result<(), io::Error> {
        match self {
            Maze::MaskedMaze { maze, start, end } => {
                maze.write_maze(out)
            },
            Maze::RadialMaze { maze, start, end } => {
                todo!();
            },
        }
    }
}
use std::{collections::HashSet, time::Instant, io, fs::File};

use cli::{Source, Destination, Command};
use color_gradients::{fire_colors, trans_colors};
use dijkstra::{Distances, DijkstraPad};
use grid::BinaryTreeSettings;
use lerp::multi_lerp;
use masked_grid::MaskedGrid;
use maze::{Maze, Algorithm};
use polar_grid::{PolarGrid, RingProfile};
use rand::{seq::SliceRandom, random, rngs::ThreadRng, distributions::Uniform, prelude::Distribution, thread_rng};
use tiny_skia::{Paint, Color, Pixmap, PremultipliedColorU8};

pub mod pool;
pub mod grid;
pub mod dijkstra;
pub mod lerp;
pub mod color_gradients;
pub mod easing;
pub mod masked_grid;
pub mod cli;
pub mod polar_grid;
pub mod geometry;
pub mod maze;
pub mod parsers;
pub mod triangle_grid;

use crate::{grid::FlatSquareGrid, polar_grid::RingPosition};

fn disk_mask(width: usize, height: usize, radius_ratio: f64, row: usize, col: usize) -> bool {
    let x = col as f64;
    let y = row as f64;
    let hc = height as f64 / 2.0;
    let wc = width as f64 / 2.0;
    let dx = wc - x;
    let dy = hc - y;
    let dist = (dx * dx + dy * dy).sqrt();
    return dist < hc.min(wc) * radius_ratio;
}

fn stripes_mask(width: usize, _height: usize, strip_width: usize, row: usize, col: usize) -> bool {
    (width - row + col ) % (strip_width) < strip_width / 2
}

/// Samples an element of the slice, with equal probability each
/// 
/// # Panics
/// 
/// Panics if `slice` is empty
pub fn sample_uniform<'s, A>(slice: &'s[A], rng: &mut ThreadRng) -> &'s A {
    &slice[Uniform::from(0..slice.len()).sample(rng)]
}

fn main() {
    let mut rng = thread_rng();
    let command = cli::CommandBuilder::new()
        .source(Source::mazefile("owo.maze"))
        .destination(Destination::image(800, 8, "output.png"))
        .build().unwrap();


    let g = match command.source {
        Source::Mazefile { input } => {
            let g = MaskedGrid::read_maze(File::open(input).unwrap()).unwrap();
            let fp = g.pool.furthest_pair().unwrap();
            Maze::MaskedMaze { maze: g, start: fp.0, end: fp.1 }
        },
        Source::FromInputMask { input } => {
            let mask_image = Pixmap::load_png(input).unwrap();
            let width = mask_image.width() as usize;
            let height = mask_image.height() as usize;
            let mask_function = move |row, col| {
                mask_image.pixel(col as u32, row as u32).unwrap() == PremultipliedColorU8::from_rgba(0,0,0,u8::MAX).unwrap()
            };
            Maze::new_masked_cartesian(width, height, Box::new(mask_function), Algorithm::AldousBroder, &mut rng)
        },
        Source::Unmasked { width, height } => {
            Maze::new_unmasked_cartesian(width, height, Algorithm::HuntAndKill, &mut rng)
        },
        Source::UnmaskedRadial { starting_branch_count, ring_count } => {
            let g = Maze::new_unmasked_radial(starting_branch_count, ring_count, Algorithm::AldousBroder, &mut rng);
            g
        }
    };

    match command.destination {
        Destination::Mazefile { output } => {
            g.write_maze(File::create(output).unwrap()).unwrap()
        },
        Destination::Image { image_width, padding, output } => {
            g.print_image(image_width, padding).save_png(output).unwrap()
        },
    }
}

pub fn get_mut_2<T>(v: &mut Vec<T>, index_1: usize, index_2: usize) -> (&mut T, &mut T) {
    assert_ne!(index_1, index_2);
    if index_2 > index_1 {
        let (l, r) = v.split_at_mut(index_2);
        return (&mut l[index_1], &mut r[0])
    } else {
        let (l, r) = v.split_at_mut(index_1);
        return (&mut l[index_2], &mut r[0])
    }
}
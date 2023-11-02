use std::{f64::consts::PI, ops::Index, fmt::Display};

use tiny_skia::{Pixmap, Paint, Stroke, LineCap, LineJoin, PathBuilder, Transform, FillRule, Path, Color};

use crate::{pool::{Pool, NodeId}, geometry::{CartesianPoint, PolarPoint}, dijkstra::{DijkstraPad, Distance}};



pub fn circumference(radius: f64) -> f64 {
    radius * PI * 2.0
}

/// Represents a sector of an annulus by polar coordinate points
pub struct SixPointArc {
    bottom_left: PolarPoint,
    bottom_center: PolarPoint,
    bottom_right: PolarPoint,
    top_left: PolarPoint,
    top_center: PolarPoint,
    top_right: PolarPoint
}

#[derive(Debug)]
pub struct RingProfile(usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RingPosition {
    pub ring: usize,
    pub column: usize,
}

pub enum RingStep {
    UpSplitLeft,
    UpSplitRight,
    UpSingle,
    CW,
    CCW,
    Down,
}

pub enum AnyAbove {
    SplitCenter(Vec<RingPosition>),
    Split(RingPosition, RingPosition),
    Single(RingPosition)
}

impl RingProfile {
    pub fn new(starting_branch_count: usize) -> Self {
        assert!(starting_branch_count > 1);
        RingProfile(starting_branch_count)
    }

    pub fn ring_cell_count(&self, ring: usize) -> usize {
        if ring == 0 {
            return 1;
        } else if ring == 1 {
            return self.0;
        }
        let mut cell_count = self.0;
        for r in 1..ring {
            if circumference((r + 1) as f64) / cell_count as f64 > 2.0 {
                cell_count *= 2;
            }
        }
        cell_count
    }

    pub fn six_point_arc(&self, pos: RingPosition) -> SixPointArc {
        let inner_radius = pos.ring as f64;
        let outer_radius = (pos.ring + 1) as f64;
        let grid_width = self.ring_cell_count(pos.ring) as f64;
        let left_angle = pos.column as f64 / grid_width * 2.0 * PI;
        let right_angle = (pos.column + 1) as f64 / grid_width * 2.0 * PI;
        let center_angle = (left_angle + right_angle) / 2.0;

        SixPointArc { 
            bottom_left: PolarPoint::new(inner_radius, left_angle), 
            bottom_center: PolarPoint::new(inner_radius, center_angle), 
            bottom_right: PolarPoint::new(inner_radius, right_angle), 
            top_left: PolarPoint::new(outer_radius, left_angle), 
            top_center: PolarPoint::new(outer_radius, center_angle), 
            top_right: PolarPoint::new(outer_radius, right_angle) 
        }
    }

    pub fn any_above(&self, pos: RingPosition) -> AnyAbove {
        let ring_width = self.ring_cell_count(pos.ring);
        let ring_width_above = self.ring_cell_count(pos.ring + 1);
        if pos.ring == 0 {
            let mut aboves = vec![];
            for column in 0..ring_width_above {
                aboves.push(RingPosition{ ring: 1, column });
            }
            AnyAbove::SplitCenter(aboves)
        } else if ring_width_above > ring_width {
            AnyAbove::Split(
                RingPosition{ column: pos.column * 2, ring: pos.ring + 1 },
                RingPosition{ column: pos.column * 2 + 1, ring: pos.ring + 1 }
            )
        } else {
            AnyAbove::Single(RingPosition{ column: pos.column, ring: pos.ring + 1 })
        }
    }

    pub fn take_step(&self, pos: RingPosition, step: RingStep) -> Option<RingPosition> {
        if pos.ring == 0 {
            return None;
        }
        match step {
            RingStep::UpSplitLeft => {
                let ring_width = self.ring_cell_count(pos.ring);
                let ring_width_above = self.ring_cell_count(pos.ring + 1);
                if ring_width_above > ring_width {
                    Some(RingPosition{ ring: pos.ring + 1, column: pos.column * 2 })
                } else {
                    None
                }
            },
            RingStep::UpSplitRight => {
                let ring_width = self.ring_cell_count(pos.ring);
                let ring_width_above = self.ring_cell_count(pos.ring + 1);
                if ring_width_above > ring_width {
                    Some(RingPosition{ ring: pos.ring + 1, column: pos.column * 2 + 1 })
                } else {
                    None
                }
            },
            RingStep::UpSingle => {
                let ring_width = self.ring_cell_count(pos.ring);
                let ring_width_above = self.ring_cell_count(pos.ring + 1);
                if ring_width == ring_width_above {
                    Some(RingPosition{ ring: pos.ring + 1, ..pos})
                } else {
                    None
                }
            },
            RingStep::CW => {
                let ring_width = self.ring_cell_count(pos.ring);
                if pos.column + 1 == ring_width {
                    Some(RingPosition{ column: 0, ..pos})
                } else {
                    Some(RingPosition{ column: pos.column + 1, ..pos })
                }
            },
            RingStep::CCW => {
                let ring_width = self.ring_cell_count(pos.ring);
                if pos.column == 0 {
                    Some(RingPosition{ column: ring_width - 1, ..pos })
                } else {
                    Some(RingPosition{ column: pos.column - 1, ..pos })
                }
            },
            RingStep::Down => {
                if pos.ring == 0 {
                    return None;
                } else if pos.ring == 1 {
                    return Some(RingPosition{ ring: pos.ring - 1, column: 0 });
                }
                let ring_width = self.ring_cell_count(pos.ring);
                let ring_width_below = self.ring_cell_count(pos.ring - 1);
                if ring_width_below < ring_width {
                    return Some(RingPosition{ ring: pos.ring - 1, column: pos.column / 2 });
                } else {
                    return Some(RingPosition{ ring: pos.ring - 1, column: pos.column });
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct PolarGrid {
    pub profile: RingProfile,
    pub pool: Pool<()>,
    pub rings: Vec<Vec<NodeId>>,
}

impl Display for PolarGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Branching factor: {}", self.profile.0)?;
        writeln!(f, "Rings: {:#?}", self.rings)?;
        writeln!(f, "Pool: {}", self.pool)
    }
}

impl Index<RingPosition> for PolarGrid {
    type Output = NodeId;

    fn index(&self, index: RingPosition) -> &Self::Output {
        &self.rings[index.ring][index.column]
    }
}

impl PolarGrid {
    pub fn new(starting_branch_count: usize, ring_count: usize) -> Self {
        let profile = RingProfile::new(starting_branch_count);
        let mut pool = Pool::new();
        let mut rings = vec![];
        for ring in 0..ring_count {
            let ring_width = profile.ring_cell_count(ring);
            rings.push(vec![]);
            for _col in 0..ring_width {
                rings[ring].push(pool.new_node(|_| ()));
            }
        }
        let mut grid = PolarGrid{ profile, pool, rings };

        assert!(ring_count > 1);

        //Stitch ring 1 to the center
        for column in 0..grid.rings[1].len() {
            grid.pool.make_adjacent(grid.rings[0][0], grid.rings[1][column], true);
        }

        //Stitch rings 1 and above to those above it BUT NOT THE OUTER ONE and ALSO stitch around the ring itself
        for ring in 1..(ring_count - 1) {
            for column in 0..grid.rings[ring].len() {
                let here = RingPosition{ ring, column };
                //Stitch cell to the one next to it
                grid.pool.make_adjacent(grid[here],
                    grid[grid.profile.take_step(here, RingStep::CW).unwrap()],
                    true
                );

                //Stitch cell to the ones above it
                match grid.profile.any_above(RingPosition{ ring, column }) {
                    AnyAbove::SplitCenter(_) => unreachable!("Cannot happen for rings >= 1"),
                    AnyAbove::Split(left, right) => {
                        grid.pool.make_adjacent(
                            grid.rings[ring][column],
                            grid[left],
                            true
                        );

                        grid.pool.make_adjacent(
                            grid.rings[ring][column],
                            grid[right],
                            true,
                        );
                    },
                    AnyAbove::Single(above) => {
                        grid.pool.make_adjacent(
                            grid.rings[ring][column],
                            grid[above],
                            true,
                        );
                    },
                }
            }
        }




        // The only thing left to stitch is the horizontal adjacencies in the final ring
        for column in 0..grid.rings[ring_count - 1].len() {
            let here = RingPosition{ ring: ring_count - 1, column };
            let there = grid.profile.take_step(here, RingStep::CW).unwrap();
            grid.pool.make_adjacent(grid[here], 
                grid[there],
                true
            );
        }

        grid
    }

    pub fn is_floor(&self, pos: RingPosition) -> bool {
        if pos.ring == 0 {
            false
        } else {
            !self.pool.is_linked(self[pos], self[self.profile.take_step(pos, RingStep::Down).unwrap()])
        }
    }

    pub fn is_left_wall(&self, pos: RingPosition) -> bool {
        if pos.ring == 0 {
            false
        } else {
            !self.pool.is_linked(self[pos], self[self.profile.take_step(pos, RingStep::CCW).unwrap()])
        }
    }

    pub fn print_image(&self, radius: usize, padding: usize, paint_function: impl Fn(NodeId) -> Paint<'static>) -> Pixmap {
        //For now only print the "cup" of rings 1 and greater
        let mut pixmap = Pixmap::new(2 * (radius + padding) as u32, 2 * (radius + padding) as u32).unwrap();
        let center = (radius + padding) as f32;
        let black = {
            let mut paint = Paint::default();
            paint.set_color_rgba8(0,0,0, u8::MAX);
            paint.anti_alias = true;
            paint
        };
        let white = {
            let mut paint = Paint::default();
            paint.set_color_rgba8(u8::MAX, u8::MAX, u8::MAX, u8::MAX);
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

        let thinner_stroke = {
            let mut stroke = Stroke::default();
            stroke.width = 1.5;
            stroke.line_cap = LineCap::Round;
            stroke.line_join = LineJoin::Round;
            stroke
        };

        let path = {
            let mut pb = PathBuilder::new();
            let ring_radius = radius as f64 / self.rings.len() as f64;
            for ring in 1..self.rings.len() {
                for column in 0..self.rings[ring].len() {
                    let arc = self.profile.six_point_arc(RingPosition{ ring, column });
                    let bl = CartesianPoint::from(arc.bottom_left * ring_radius);
                    let bc = CartesianPoint::from(arc.bottom_center * ring_radius);
                    let br = CartesianPoint::from(arc.bottom_right * ring_radius);
                    let tl = CartesianPoint::from(arc.top_left * ring_radius);
                    let tc = CartesianPoint::from(arc.top_center * ring_radius);
                    let tr = CartesianPoint::from(arc.top_right * ring_radius);

                    /*
                    let here = RingPosition{ ring, column };
                    if let Some(left_pos) = self.profile.take_step(here, RingStep::CCW) {
                        if self.pool.is_linked(self[here], self[left_pos]) {
                            pb.move_to(arc_center.x as f32, arc_center.y as f32);
                            pb.line_to(left.x as f32, left.y as f32);
                        }
                    }
                    if let Some(right_pos) = self.profile.take_step(here, RingStep::CW) {
                        if self.pool.is_linked(self[here], self[right_pos]) {
                            pb.move_to(arc_center.x as f32, arc_center.y as f32);
                            pb.line_to(right.x as f32, right.y as f32);
                        }
                    }
                    if let Some(down_pos) = self.profile.take_step(here, RingStep::Down) {
                        if self.pool.is_linked(self[here], self[down_pos]) {
                            pb.move_to(arc_center.x as f32, arc_center.y as f32);
                            pb.line_to(bc.x as f32, bc.y as f32);
                        }
                    }
                    if ring != self.rings.len() - 1 {
                        match self.profile.any_above(here) {
                            AnyAbove::SplitCenter(all_aboves) => {
    
                            },
                            AnyAbove::Split(al, ar) => {
                                if self.pool.is_linked(self[here], self[al]) {
                                    pb.move_to(arc_center.x as f32, arc_center.y as f32);
                                    pb.line_to(tl.x as f32, tl.y as f32);
                                }
                                if self.pool.is_linked(self[here], self[ar]) {
                                    pb.move_to(arc_center.x as f32, arc_center.y as f32);
                                    pb.line_to(tr.x as f32, tr.y as f32);
                                }
                            },
                            AnyAbove::Single(above) => {
                                if self.pool.is_linked(self[here], self[above]) {
                                    pb.move_to(arc_center.x as f32, arc_center.y as f32);
                                    pb.line_to(tc.x as f32, tc.y as f32);
                                }
                            },
                        }
                    }*/

                    match (self.is_left_wall(RingPosition{ ring, column }), self.is_floor(RingPosition { ring, column })) {
                        (true, true) => {
                            pb.move_to(tl.x as f32, tl.y as f32);
                            pb.line_to(bl.x as f32, bl.y as f32);
                            pb.quad_to(bc.x as f32, bc.y as f32, br.x as f32, br.y as f32);
                        },
                        (true, false) => {
                            pb.move_to(tl.x as f32, tl.y as f32);
                            pb.line_to(bl.x as f32, bl.y as f32);
                        },
                        (false, true) => {
                            pb.move_to(bl.x as f32, bl.y as f32);
                            pb.quad_to(bc.x as f32, bc.y as f32, br.x as f32, br.y as f32);
                        },
                        (false, false) => {},
                    }
                    //pb.line_to(tr.0 as f32, tr.1 as f32);

                }
            }
            
            pb.push_circle(0.0, 0.0, radius as f32);

            pb.finish().unwrap()
        };

        // Paint the interior of all cells

        pixmap.fill_path(
            &PathBuilder::from_circle(0.0, 0.0, radius as f32).unwrap(),
            &white,
            FillRule::EvenOdd,
            Transform::identity().pre_translate(center, center),
            None
        );
        for ring in 1..self.rings.len() {
            for column in 0..self.rings[ring].len() {
                let ring_radius = radius as f64 / self.rings.len() as f64;
                let arc = self.profile.six_point_arc(RingPosition{ ring, column });
                let bl = CartesianPoint::from(arc.bottom_left * ring_radius);
                let bc = CartesianPoint::from(arc.bottom_center * ring_radius);
                let br = CartesianPoint::from(arc.bottom_right * ring_radius);
                let tl = CartesianPoint::from(arc.top_left * ring_radius);
                let tc = CartesianPoint::from(arc.top_center * ring_radius);
                let tr = CartesianPoint::from(arc.top_right * ring_radius);

                let cell = {
                    let mut pb = PathBuilder::new();
                    pb.move_to(tl.x as f32, tl.y as f32);
                    pb.line_to(bl.x as f32, bl.y as f32);
                    pb.quad_to(bc.x as f32, bc.y as f32, br.x as f32, br.y as f32);
                    pb.line_to(tr.x as f32, tr.y as f32);
                    pb.quad_to(tc.x as f32, tc.y as f32, tl.x as f32, tl.y as f32);
                    pb.finish().unwrap()
                };
                pixmap.stroke_path(
                    &cell,
                    &paint_function(self.rings[ring][column]),
                    &thinner_stroke,
                    Transform::identity().pre_translate(center, center),
                    None
                );
                pixmap.fill_path(
                    &cell,
                    &paint_function(self.rings[ring][column]),
                    FillRule::EvenOdd,
                    Transform::identity().pre_translate(center, center),
                    None
                );
            }
        }
        

        pixmap.stroke_path(&path, &black, &stroke, Transform::identity().pre_translate(center, center), None);
        

        pixmap
    }

    pub fn print_image_distances(&self, radius: usize, padding: usize, start_node: NodeId, color_function: impl Fn(f64) -> Color) -> Pixmap {
        let distances = DijkstraPad::new(&self.pool, start_node).perform();
        let max_finite_distance = distances.pool.payloads().map(|d| {
            match d {
                Distance::Finite(dist) => *dist,
                Distance::Infinite => 0,
            }
        }).max().unwrap_or(0) as f64;

        if max_finite_distance == 0.0 {
            self.print_image(radius, padding, |_| {
                let mut p = Paint::default();
                p.set_color_rgba8(u8::MAX, u8::MAX, u8::MAX, u8::MAX);
                p
            })
        } else {
            self.print_image(radius, padding, |node_id| {
                let dist = distances.pool.get(node_id).payload.as_finite().unwrap_or(0) as f64;
                let normalized_distance = dist / max_finite_distance;
                let mut p = Paint::default();
                p.set_color(color_function(normalized_distance));
                p
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_below() {
        let profile = RingProfile::new(6);
        let center = RingPosition{ ring: 0, column: 0 };
        assert_eq!(profile.take_step(center, RingStep::Down), None);

        let ring1: Vec<RingPosition> = (0..profile.ring_cell_count(1)).map(|col| RingPosition{ ring: 1, column: col}).collect();
        for c in ring1 {
            assert_eq!(profile.take_step(c, RingStep::Down), Some(center));
        }

        assert_eq!(profile.take_step(RingPosition{ ring: 2, column: 5 }, RingStep::Down), Some(RingPosition{ ring: 1, column: 2 }));
        assert_eq!(profile.take_step(RingPosition{ ring: 3, column: 4 }, RingStep::Down), Some(RingPosition{ ring: 2, column: 4 }));
    }

    #[test]
    fn test_cw_cww() {
        let profile = RingProfile::new(6);
        let center = RingPosition{ ring: 0, column: 0 };
        
        let ring1: Vec<RingPosition> = (0..profile.ring_cell_count(1)).map(|col| RingPosition{ ring: 1, column: col }).collect();
        for c in ring1 {
            assert_eq!(profile.take_step(c, RingStep::CW), Some(RingPosition{ ring: 1, column: (c.column + 1) % profile.ring_cell_count(1)}));
            if c.column == 0 {
                let step = profile.take_step(c, RingStep::CCW);
                let expected = Some(RingPosition{ ring: 1, column: profile.ring_cell_count(1) - 1 });
                assert_eq!(step, expected);
            } else {
                let step = profile.take_step(c, RingStep::CCW);
                let expected = Some(RingPosition{ ring: 1, column: (c.column - 1) % profile.ring_cell_count(1)});
                assert_eq!(step, expected);
            }
            
        }
    }
}
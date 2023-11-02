use std::ops::{Mul, Add, Div};



#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct PolarPoint {
    pub r: f64,
    pub theta: f64,
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct CartesianPoint { 
    pub x: f64,
    pub y: f64,
}

impl From<PolarPoint> for CartesianPoint {
    fn from(PolarPoint{ r, theta}: PolarPoint) -> Self {
        CartesianPoint{ x: r * theta.cos(), y: r * theta.sin() }
    }
}

impl PolarPoint {
    pub fn new(r: f64, theta: f64) -> Self {
        PolarPoint{ r, theta }
    }
}

impl Mul<f64> for PolarPoint {
    type Output = PolarPoint;

    fn mul(self, rhs: f64) -> Self::Output {
        PolarPoint {
            r: self.r * rhs,
            ..self
        }
    }
}

impl Add for CartesianPoint {
    type Output = CartesianPoint;

    fn add(self, rhs: Self) -> Self::Output {
        CartesianPoint { 
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Mul<f64> for CartesianPoint {
    type Output = CartesianPoint;

    fn mul(self, rhs: f64) -> Self::Output {
        CartesianPoint {
            x: self.x * rhs,
            y: self.y * rhs
        }
    }
}

impl Div<f64> for CartesianPoint {
    type Output = CartesianPoint;

    fn div(self, rhs: f64) -> Self::Output {
        CartesianPoint {
            x: self.x / rhs,
            y: self.y / rhs
        }
    }
}
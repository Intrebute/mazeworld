use tiny_skia::Color;



pub trait Lerpable {
    fn inner_lerp(start: &Self, end: &Self, t: f64) -> Self;
}

impl Lerpable for f32 {
    fn inner_lerp(start: &f32, end: &f32, t: f64) -> f32 {
        Lerpable::inner_lerp(&(*start as f64), &(*end as f64), t) as f32
    }
}

impl Lerpable for f64 {
    fn inner_lerp(start: &f64, end: &f64, t: f64) -> f64 {
        (1.0 - t) * start + t * end
    }
}

impl<F: Lerpable + Clone, const N: usize> Lerpable for [F; N] {
    fn inner_lerp(start: &[F; N], end: &[F; N], t: f64) -> [F; N] {
        let mut interps = start.clone();
        for i in 0..N {
            interps[i] = F::inner_lerp(&start[i], &end[i], t);
        }
        interps
    }
}

impl Lerpable for Color {
    fn inner_lerp(start: &Self, end: &Self, t: f64) -> Self {
        let r = Lerpable::inner_lerp(&start.red(), &end.red(), t);
        let g = Lerpable::inner_lerp(&start.green(), &end.green(), t);
        let b = Lerpable::inner_lerp(&start.blue(), &end.blue(), t);
        let a = Lerpable::inner_lerp(&start.alpha(), &end.alpha(), t);
        Color::from_rgba(r,g,b,a).unwrap()
    }
}

pub fn lerp<F: Lerpable>(start: F, end: F) -> impl Fn(f64) -> F {
    move |t| {
        F::inner_lerp(&start, &end, t)
    }
}

pub fn multi_lerp<F: Lerpable + Clone, const N: usize>(points: [F; N]) -> impl Fn(f64) -> F {
    assert_ne!(N, 0);
    move |t| {
        if N == 1 {
            points[0].clone()
        } else {
            let segment: usize = (t * (N - 1) as f64).floor() as usize;
            F::inner_lerp(&points[segment], &points[(segment + 1).min(N - 1)], (t * (N - 1) as f64).fract())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn e_test(a: f64, b: f64) -> bool {
        a - b < 1e-10
    }

    #[test]
    fn lerp_test() {
        let l = lerp(0.0, 100.0);
        assert!(e_test(l(0.0), 0.0));
        assert!(e_test(l(0.5), 50.0));
        assert!(e_test(l(1.0), 100.0));
    }

    #[test]
    fn lerp_negatives() {
        let l = lerp(0.0, -100.0);
        assert!(e_test(l(0.0), 0.0));
        assert!(e_test(l(0.5), -50.0));
        assert!(e_test(l(1.0), -100.0));
    }

    #[test]
    fn multi_lerp_one_point() {
        let l = multi_lerp([100.0]);
        assert!(e_test(l(0.0), 100.0));
        assert!(e_test(l(0.5), 100.0));
        assert!(e_test(l(1.0), 100.0));
    }

    #[test]
    fn multi_lerp_two_points() {
        let l = multi_lerp([100.0, 0.0]);
        assert!(e_test(l(0.0), 100.0));
        assert!(e_test(l(0.5), 50.0));
        assert!(e_test(l(1.0), 0.0));
    }

    #[test]
    fn multi_lerp_more_points() {
        let l = multi_lerp([0.0, 100.0, 50.0]);
        assert!(e_test(l(0.0), 0.0));
        assert!(e_test(l(0.25), 50.0));
        assert!(e_test(l(0.5), 100.0));
        assert!(e_test(l(0.75), 75.0));
        assert!(e_test(l(1.0), 50.0));
    }
}
// ax^2 + bx + c
#[derive(Debug)]
pub struct Parabola {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Parabola {
    pub fn from_line_and_point(x: f64, y: f64, y_line: f64) -> Parabola {
        let d = y - y_line;

        Parabola {
            a: 0.5 / d,
            b: -x / d,
            c: (x.powi(2) + y.powi(2) - y_line.powi(2)) * 0.5 / d
        }
    }

    pub fn solve(&self) -> (f64, f64) {
        let delta = self.b.powi(2) - 4.0 * self.a * self.c;

        if delta < 0.0 {
            (f64::NAN, f64::NAN)
        } else {
            let sqrt_delta = delta.sqrt();
            (
                (-self.b + sqrt_delta) * 0.5 / self.a,
                (-self.b - sqrt_delta) * 0.5 / self.a
            )
        }
    }

    pub fn eval(&self, x: f64) -> f64 {
        self.a * x * x + self.b * x + self.c
    }

    pub fn collides_at<F>(&self, other: &Parabola, func: F) -> bool
    where
        F: Fn(f64) -> bool
    {
       let collisions = Parabola { 
           a: self.a - other.a,
           b: self.b - other.b,
           c: self.c - other.c
        }.solve();

        collisions.0 != f64::NAN && (func(collisions.0) || func(collisions.1))
    }
}

// struct Beachline<const L: usize> {
//     parabolas: [Parabola; L]
// }

// impl<const L: usize> Beachline<L> {
//     pub fn eval(&self, x: f64) -> f64 {
//         let min = f64::INFINITY;
//         for p in self.parabolas {
//             let eval = p.eval(x);
//             if eval < min {
//                 min = eval
//             }
//         }

//         min
//     }
// }

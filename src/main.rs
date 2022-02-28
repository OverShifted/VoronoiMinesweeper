use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

mod parabola;
use crate::parabola::*;

const W: u32 = 1800;
const H: u32 = 900;
const C: u32 = 300;

#[derive(Clone, Copy)]
struct Point {
    id: u32,
    x: u32,
    y: u32,
    c: Color
}

fn gen_points() -> Vec<Point> {
    let mut points = Vec::new();

    for i in 0..C {
        points.push(Point {
            id: i,
            x: (rand::random::<f64>() * W as f64) as u32,
            y: (rand::random::<f64>() * H as f64) as u32,
            c: Color::RGB(rand::random::<u8>(), rand::random::<u8>(), rand::random::<u8>())
        });
    }

    points
}

fn get_point_cell(x: u32, y: u32, points: &Vec<Point>) -> (Point, f64) {
    let mut best_dist = f64::INFINITY;
    let mut best_point = Point { id: 0, x: 0, y: 0, c: Color::BLACK };

    for point in points {
        let d = (x as f64 - point.x as f64).powf(2.0) + (y as f64 - point.y as f64).powf(2.0);
        if d < best_dist {
            best_dist = d;
            best_point = *point;
        }
    }

    (best_point, best_dist)
}

fn are_neighbor(p0: &Point, p1: &Point, points: &Vec<Point>) -> bool {
    let mid_x = (p0.x + p1.x) / 2;
    let mid_y = (p0.y + p1.y) / 2;

    let (Point { id, .. }, _) = get_point_cell(mid_x, mid_y, points);
    id == p0.id || id == p1.id
}

fn main() {
    // println!("{:?}", crate::parabola::Parabola{ a: 1.0, b: 1.0, c: 1.0 }.solve());

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let window = video_subsystem.window("> /dev/null", W, H)
        .position_centered()
        .build()
        .unwrap();

    let points = gen_points();

    {
        let mut points_left: Vec<Option<Point>> = points.iter().map(|&p| Some(p)).collect();
        let mut points_saw = Vec::new();

        for sweepline in 0..H {
            // Add new points
            for p_option in &mut points_left {
                if let Some(p) = *p_option {
                    if p.y == sweepline {
                        points_saw.push(p);
                        println!("{}", p.y);
                        *p_option = None;
                    }
                }
            }

            let parabolas: Vec<Parabola> = points_saw.iter().map(|&p| Parabola::from_line_and_point(p.x as f64, p.y as f64, sweepline as f64)).collect();

            let eval_beachline = |x| {
                let mut min = f64::INFINITY;
                for p in &parabolas {
                    let eval = p.eval(x);
                    if eval < min {
                        min = eval
                    }
                }

                min
            };

            for p0 in &parabolas {
                for p1 in &parabolas {
                    p0.collides_at(p1, |x| p0.eval(x) < eval_beachline(x));
                }
            }
        }
    }

    'running: loop {

        let mut surface = window.surface(&event_pump).unwrap();
        
        surface.with_lock_mut(|data| {
            for x in 0..W {
                for y in 0..H {
                    let (Point { c: mut color, .. }, best_dist) = get_point_cell(x, y, &points);

                    if best_dist < 10.0 {
                        color = Color::BLACK;
                    }

                    data[(4 * (x + W * y) + 0) as usize] = color.r;
                    data[(4 * (x + W * y) + 1) as usize] = color.g;
                    data[(4 * (x + W * y) + 2) as usize] = color.b;
                    data[(4 * (x + W * y) + 3) as usize] = 255;
                }
            }
        });

        surface.finish().unwrap();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
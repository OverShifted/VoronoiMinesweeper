use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::time::Duration;
use sdl2::gfx::primitives::DrawRenderer;

use voronator::{VoronoiDiagram, delaunator::Point as VorPoint};

const W: u32 = 1800;
const H: u32 = 900;
const C: usize = 300;

#[derive(Clone, Copy, Debug)]
struct Point {
    x: u32,
    y: u32,
    bomb: bool,
    covered: bool,
    flagged: bool,
    neighbor_bombs: u32
}

fn get_point_cell(x: u32, y: u32, points: &Vec<Point>) -> (usize, f64) {
    let mut best_dist = f64::INFINITY;
    let mut best_point = 0;

    for (i, point) in points.iter().enumerate() {
        let d = (x as f64 - point.x as f64).powf(2.0) + (y as f64 - point.y as f64).powf(2.0);
        if d < best_dist {
            best_dist = d;
            best_point = i;
        }
    }

    (best_point, best_dist)
}

fn calculate_bomb_neighbours(diagram: &VoronoiDiagram::<VorPoint>, points: &mut Vec<Point>) {
    for i in 0..C {
        let neighbor = &diagram.neighbors[i];
        let bomb_count: u32 = neighbor.iter().map(|&n| if n < C && points[n].bomb { 1 } else { 0 }).sum();
        points[i].neighbor_bombs = bomb_count;
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let window = video_subsystem.window("> /dev/null", W, H)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut points: Vec<Point> = (0..C)
        .map(|_| Point {
            x: (rand::random::<f64>() * W as f64) as u32,
            y: (rand::random::<f64>() * H as f64) as u32,
            bomb: rand::random::<bool>(),
            covered: true,
            flagged: false,
            neighbor_bombs: 0
        }).collect();

    let diagram = VoronoiDiagram::<VorPoint>::from_tuple(
        &(0., 0.), &(W as f64, H as f64),
        &points.iter().map(|p| (p.x as f64, p.y as f64)).collect::<Vec<(f64, f64)>>()
    ).unwrap();

    let mut player_is_looser = false;
    let mut all_covered = true;

    'running: loop {

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::MouseButtonDown{ mouse_btn: MouseButton::Left, x, y, .. } => {
                    if !player_is_looser {
                        let (i, _) = get_point_cell(x as u32, y as u32, &points);

                        if all_covered {
                            if points[i].bomb {
                                for point in &mut points {
                                    point.bomb = !point.bomb;
                                }
                            }
                            calculate_bomb_neighbours(&diagram, &mut points);
                        }

                        all_covered = false;

                        let point = &mut points[i];
                        point.covered = false;

                        if point.bomb {
                            println!("Boom :)))");
                            // player_is_looser = true;
                        } else {
                            // Uncover all non-bomb neighbor cells
                            println!("This: {}", points[i].neighbor_bombs);
                            let mut queue = vec![Some(i)];
                            let mut i = 0;

                            'queue: loop {
                                if let Some(cell) = queue[i] {
                                    let neighbors = &diagram.neighbors[cell];

                                    for &neighbor_idx in neighbors {
                                        if neighbor_idx >= C as usize {
                                            continue
                                        }

                                        let neighbor = &mut points[neighbor_idx];

                                        if !neighbor.bomb && neighbor.covered {
                                            neighbor.covered = false;
                                            queue.push(Some(neighbor_idx));
                                        }
                                    }

                                    queue[i] = None;
                                } else {
                                    i += 1;
                                    if i == queue.len() {
                                        break 'queue;
                                    }
                                }
                            }
                        }
                    }
                },
                Event::MouseButtonDown{ mouse_btn: MouseButton::Right, x, y, .. } => {
                    if !player_is_looser {
                        let (i, _) = get_point_cell(x as u32, y as u32, &points);

                        points[i].flagged = !points[i].flagged;
                    }
                },
                _ => {}
            }
        }


        for (i, cell) in diagram.cells().iter().enumerate() {
            let xs: Vec<i16> = cell.points().into_iter()
                .map(|p| p.x as i16)
                .collect();

            let ys: Vec<i16> = cell.points().into_iter()
                .map(|p| p.y as i16)
                .collect();

            let err = canvas.filled_polygon(&xs, &ys,
                if points[i].covered {
                    if points[i].flagged {
                        Color::MAGENTA
                    } else {
                        Color::WHITE
                    }
                } else {
                    if points[i].bomb {
                        Color::RED
                    } else {
                        Color::GRAY
                    }
                }
            );//.unwrap();

            if let Err(..) = err {
                println!("Error at cell {} {:?}", i, points[i]);
            }
        }

        for (i, cell) in diagram.cells().iter().enumerate() {
            let xs: Vec<i16> = cell.points().into_iter()
                .map(|p| p.x as i16)
                .collect();

            let ys: Vec<i16> = cell.points().into_iter()
                .map(|p| p.y as i16)
                .collect();

            let err = canvas.aa_polygon(&xs, &ys, Color::BLACK);

            if let Err(..) = err {
                println!("Error at cell {} {:?}", i, points[i]);
            }

            let point = points[i];

            if !point.covered && !point.bomb {
                canvas.string(points[i].x as i16, points[i].y as i16, &points[i].neighbor_bombs.to_string(), Color::BLUE).unwrap();
            }
        }

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

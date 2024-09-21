use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, TextureCreator};
use std::time::Duration;

use voronator::{delaunator::Point as VorPoint, VoronoiDiagram};

mod colors;

const W: u32 = 1800;
const H: u32 = 900;

const CX: usize = 10;
const CY: usize = 5;
const C: usize = CX * CY;

const PAD: usize = 100;

#[derive(Clone, Copy, Debug)]
struct Point {
    x: u32,
    y: u32,
    bomb: bool,
    covered: bool,
    flagged: bool,
    neighbor_bombs: u32,
}

impl Point {
    fn color(&self) -> Color {
        match self.covered {
            true => match self.flagged {
                true => colors::CELL_FLAGGED,
                false => colors::CELL_COVERED,
            },
            false => match self.bomb {
                true => colors::CELL_BOMBED,
                false => colors::CELL_UNCOVERED,
            },
        }
    }
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

fn calculate_bomb_neighbours(diagram: &VoronoiDiagram<VorPoint>, points: &mut Vec<Point>) {
    for i in 0..C {
        let neighbor = &diagram.neighbors[i];
        let bomb_count: u32 = neighbor
            .iter()
            .map(|&n| if n < C && points[n].bomb { 1 } else { 0 })
            .sum();
        points[i].neighbor_bombs = bomb_count;
    }
}

fn get_random_bombness() -> bool {
    rand::random::<f64>() > 0.85
}

fn check_win(points: &Vec<Point>) -> bool {
    for point in points {
        if point.flagged != point.bomb {
            return false;
        }
    }

    true
}

fn render_text<Ctx, RTarget: RenderTarget>(
    texture_creator: &TextureCreator<Ctx>,
    canvas: &mut Canvas<RTarget>,
    center: (i32, i32),
    string: &str,
    color: Color,
) {
    let context = sdl2::ttf::init().unwrap();
    let font = context.load_font("Roboto/Roboto-Regular.ttf", 30).unwrap();
    let surface = font.render(string).blended(color).unwrap();
    let texture = surface.as_texture(texture_creator).unwrap();

    canvas
        .copy(
            &texture,
            None,
            Rect::from_center(
                sdl2::rect::Point::new(center.0, center.1),
                surface.size().0,
                surface.size().1,
            ),
        )
        .unwrap();
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let window = video_subsystem
        .window("Voronoi minesweeper", W, H)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut points = Vec::new();

    for x in 0..CX {
        for y in 0..CY {
            points.push(Point {
                x: (x as f64 / CX as f64 * (W as f64 - 2.0 * PAD as f64)
                    + PAD as f64
                    + (rand::random::<f64>() - 0.5) * 200.0 as f64) as u32,
                y: (y as f64 / CY as f64 * (H as f64 - 2.0 * PAD as f64)
                    + PAD as f64
                    + (rand::random::<f64>() - 0.5) * 200.0 as f64) as u32,
                bomb: get_random_bombness(),
                covered: true,
                flagged: false,
                neighbor_bombs: 0,
            });
        }
    }

    // println!("{:?}", points);
    // println!("{:?}", (0..CX).zip(0..CY).collect::<Vec<(usize, usize)>>());

    let diagram = VoronoiDiagram::<VorPoint>::from_tuple(
        &(0., 0.),
        &(W as f64, H as f64),
        &points
            .iter()
            .map(|p| (p.x as f64, p.y as f64))
            .collect::<Vec<(f64, f64)>>(),
    )
    .unwrap();

    let mut game_over = false;
    let mut all_covered = true;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    if !game_over {
                        let (i, _) = get_point_cell(x as u32, y as u32, &points);

                        if all_covered {
                            // If we hit a bomb on the first try,
                            // pretend that it wasn't a bomb in the first place.
                            // This is why calculate_bomb_neighbours is defered until here.
                            points[i].bomb = false;
                            calculate_bomb_neighbours(&diagram, &mut points);
                            all_covered = false;
                        }

                        let point = &mut points[i];
                        point.covered = false;

                        if point.bomb {
                            println!("Boom :)))");
                            game_over = true;
                        } else {
                            // Uncover all non-bomb neighbor cells
                            // println!("This: {}", points[i].neighbor_bombs);
                            // TODO: Is Option<T> really needed?
                            let mut queue = vec![Some(i)];
                            let mut i = 0;

                            'queue: loop {
                                if let Some(cell) = queue[i] {
                                    let neighbors = &diagram.neighbors[cell];

                                    for &neighbor_idx in neighbors {
                                        if neighbor_idx >= C as usize {
                                            continue;
                                        }

                                        let neighbor = &mut points[neighbor_idx];

                                        if !neighbor.bomb && !neighbor.flagged && neighbor.covered {
                                            neighbor.covered = false;

                                            if neighbor.neighbor_bombs == 0 {
                                                queue.push(Some(neighbor_idx));
                                            }
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
                }
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Right,
                    x,
                    y,
                    ..
                } => {
                    if !game_over {
                        let (i, _) = get_point_cell(x as u32, y as u32, &points);

                        points[i].flagged = !points[i].flagged;
                        if check_win(&points) {
                            println!("Won :)");
                            game_over = true;
                        }
                    }
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        // Render cells
        for (i, cell) in diagram.cells().iter().enumerate() {
            let xs: Vec<i16> = cell.points().into_iter().map(|p| p.x as i16).collect();

            let ys: Vec<i16> = cell.points().into_iter().map(|p| p.y as i16).collect();

            let result = canvas.filled_polygon(&xs, &ys, points[i].color());
            if let Err(..) = result {
                println!("Error at cell {} {:?}", i, points[i]);
            }
        }

        // Draw black outlines and numbers
        for (i, cell) in diagram.cells().iter().enumerate() {
            let xs: Vec<i16> = cell.points().into_iter().map(|p| p.x as i16).collect();

            let ys: Vec<i16> = cell.points().into_iter().map(|p| p.y as i16).collect();

            let result = canvas.aa_polygon(&xs, &ys, colors::CELL_OUTLINE);
            if let Err(..) = result {
                println!("Error at cell {} {:?}", i, points[i]);
            }

            let point = &points[i];
            if !point.covered && !point.bomb && point.neighbor_bombs > 0 {
                let mut mean = (0.0, 0.0);

                for point in cell.points() {
                    mean.0 += point.x;
                    mean.1 += point.y;
                }
                mean.0 /= cell.points().len() as f64;
                mean.1 /= cell.points().len() as f64;

                let mean = (mean.0 as i32, mean.1 as i32);
                render_text(
                    &texture_creator,
                    &mut canvas,
                    mean,
                    &point.neighbor_bombs.to_string(),
                    colors::NUMBERS,
                );
            }
        }

        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

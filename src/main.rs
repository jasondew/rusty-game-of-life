use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const SCALE: f32 = 10.0;
const CYCLES_TO_DIE: usize = 8;

#[derive(Clone)]
enum CellState {
    Alive,
    Dying(usize),
    Dead,
}

#[derive(Clone)]
struct Cell {
    state: CellState,
    neighbor_count: usize,
}

struct Board {
    generation: usize,
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Cell {
    pub fn dead() -> Self {
        Cell {
            state: CellState::Dead,
            neighbor_count: 0,
        }
    }

    pub fn alive() -> Self {
        Cell {
            state: CellState::Alive,
            neighbor_count: 0,
        }
    }
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![Cell::dead(); width * height];
        let mut board = Self {
            generation: 0,
            width: width,
            height: height,
            cells: cells,
        };

        board.add_glider_gun();

        board
    }

    pub fn step(self: &mut Self) {
        for cell in self.cells.iter_mut() {
            match cell.state {
                CellState::Alive => {
                    if cell.neighbor_count < 2 || cell.neighbor_count > 3 {
                        cell.state = CellState::Dying(CYCLES_TO_DIE)
                    }
                }
                CellState::Dying(cycles_left) => {
                    if cell.neighbor_count == 3 {
                        cell.state = CellState::Alive
                    } else {
                        if cycles_left == 0 {
                            cell.state = CellState::Dead
                        } else {
                            cell.state = CellState::Dying(cycles_left - 1)
                        }
                    }
                }
                CellState::Dead => {
                    if cell.neighbor_count == 3 {
                        cell.state = CellState::Alive
                    }
                }
            }
        }
    }

    pub fn update_live_neighbor_counts(self: &mut Self) {
        let neighbor_counts: Vec<usize> = self
            .cells
            .iter()
            .enumerate()
            .map(|(index, _cell)| self.live_neighbor_count(index))
            .collect();

        for (index, cell) in self.cells.iter_mut().enumerate() {
            cell.neighbor_count = neighbor_counts[index];
        }
    }

    pub fn index_to_coordinates(self: &Self, index: usize) -> (i32, i32) {
        let x = index.wrapping_rem(self.width) as i32;
        let y = index.wrapping_div(self.width) as i32;

        return (x, y);
    }

    fn coordinates_to_index(self: &Self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || x >= (self.width as i32) {
            return None;
        }

        if y < 0 || y >= (self.height as i32) {
            return None;
        }

        return Some((y as usize) * self.width + (x as usize));
    }

    fn live_neighbor_count(self: &Self, index: usize) -> usize {
        let (x, y) = self.index_to_coordinates(index);
        let cell_indices = vec![
            self.coordinates_to_index(x - 1, y - 1),
            self.coordinates_to_index(x, y - 1),
            self.coordinates_to_index(x + 1, y - 1),
            self.coordinates_to_index(x - 1, y),
            self.coordinates_to_index(x + 1, y),
            self.coordinates_to_index(x - 1, y + 1),
            self.coordinates_to_index(x, y + 1),
            self.coordinates_to_index(x + 1, y + 1),
        ];

        return cell_indices
            .iter()
            .filter(|maybe_index| match maybe_index {
                Some(index) => match self.cells[*index].state {
                    CellState::Alive => true,
                    _ => false,
                },
                None => false,
            })
            .count();
    }

    fn add_glider_gun(self: &mut Self) {
        for (x, y) in [
            (25, 1),
            (23, 2),
            (25, 2),
            (13, 3),
            (14, 3),
            (21, 3),
            (22, 3),
            (35, 3),
            (36, 3),
            (12, 4),
            (16, 4),
            (21, 4),
            (22, 4),
            (35, 4),
            (36, 4),
            (1, 5),
            (2, 5),
            (11, 5),
            (17, 5),
            (21, 5),
            (22, 5),
            (1, 6),
            (2, 6),
            (11, 6),
            (15, 6),
            (17, 6),
            (18, 6),
            (23, 6),
            (25, 6),
            (11, 7),
            (17, 7),
            (25, 7),
            (12, 8),
            (16, 8),
            (13, 9),
            (14, 9),
        ]
        .iter()
        {
            self.cells[x + y * self.width] = Cell::alive();
        }
    }
}

fn draw(mut canvas: &mut Canvas<Window>, board: &mut Board) {
    board.generation += 1;

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    board.update_live_neighbor_counts();
    board.step();

    for (index, cell) in board.cells.iter().enumerate() {
        match cell.state {
            CellState::Alive => {
                draw_cell(&mut canvas, board, index, Color::RGB(0, 0, 0));
            }
            CellState::Dying(cycles_left) => {
                let percent_done: f32 = (cycles_left as f32) / (CYCLES_TO_DIE as f32);
                let intensity: u8 = ((-0.25 * percent_done).exp() * 255.0) as u8;
                draw_cell(
                    &mut canvas,
                    board,
                    index,
                    Color::RGB(intensity, intensity, intensity),
                );
            }
            CellState::Dead => {}
        }
    }
}

fn draw_cell(canvas: &mut Canvas<Window>, board: &Board, index: usize, color: Color) {
    let (x, y) = board.index_to_coordinates(index);

    canvas.set_draw_color(color);
    canvas.draw_point((x, y)).expect("failed to draw pixel")
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Rusty Game of Life", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut board = Board::new(WIDTH, HEIGHT);

    canvas.set_scale(SCALE, SCALE)?;

    'running: loop {
        draw(&mut canvas, &mut board);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;

const WIDTH: usize = 600;
const HEIGHT: usize = 600;
const CYCLES_TO_DIE: usize = 25;

enum CellState {
    Alive,
    Dying(usize),
    Dead,
}

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

    pub fn alive(neighbor_count: usize) -> Self {
        Cell {
            state: CellState::Alive,
            neighbor_count: neighbor_count,
        }
    }

    pub fn dying(neighbor_count: usize, cycles_left: usize) -> Self {
        Cell {
            state: CellState::Dying(cycles_left),
            neighbor_count: neighbor_count,
        }
    }
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = vec![Cell::dead(); width * height];

        cells[10 * width + 12] = Cell::alive(0);
        cells[11 * width + 12] = Cell::alive(0);
        cells[11 * width + 10] = Cell::alive(0);
        cells[12 * width + 11] = Cell::alive(0);
        cells[12 * width + 12] = Cell::alive(0);

        Self {
            generation: 0,
            width: width,
            height: height,
            cells: cells,
        }
    }

    pub fn step(self: &mut Self) {
        for cell in self.cells.iter_mut() {
            match cell.state {
                CellState::Alive => {
                    if cell.neighbor_count < 2 || cell.neighbor_count > 3 {
                        *cell = Cell::dying(cell.neighbor_count, CYCLES_TO_DIE)
                    }
                }
                CellState::Dying(cycles_left) => {
                    if cell.neighbor_count == 3 {
                        *cell = Cell::alive(cell.neighbor_count)
                    } else {
                        if cycles_left == 0 {
                            *cell = Cell::dead()
                        } else {
                            *cell = Cell::dying(cell.neighbor_count, cycles_left - 1)
                        }
                    }
                }
                CellState::Dead => {
                    if cell.neighbor_count == 3 {
                        *cell = Cell::alive(cell.neighbor_count)
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
}

fn draw(mut canvas: &mut Canvas<Window>, board: &mut Board) {
    board.generation += 1;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    board.update_live_neighbor_counts();
    board.step();

    for (index, cell) in board.cells.iter().enumerate() {
        match cell.state {
            CellState::Alive => {
                draw_cell(&mut canvas, board, index, Color::RGB(255, 255, 255));
            }
            CellState::Dying(cycles_left) => {
                let intensity = (255.0 * 0.3 * (cycles_left as f32) / (CYCLES_TO_DIE as f32)) as u8;
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

    canvas.set_scale(16.0, 16.0)?;

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

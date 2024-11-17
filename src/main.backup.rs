use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{error::Error, io, time::{Duration, Instant}};
use rand::Rng;

struct App {
    grid: Vec<Vec<bool>>,
    width: usize,
    height: usize,
    running: bool,
}

impl App {
    fn new(width: usize, height: usize) -> App {
        let mut rng = rand::thread_rng();
        let grid = (0..height)
            .map(|_| (0..width).map(|_| rng.gen_bool(0.3)).collect())
            .collect();

        App {
            grid,
            width,
            height,
            running: false,
        }
    }

    fn update(&mut self) {
        let mut new_grid = self.grid.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                let live_neighbors = self.count_neighbors(x, y);
                let cell = self.grid[y][x];

                new_grid[y][x] = match (cell, live_neighbors) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise,
                };
            }
        }

        self.grid = new_grid;
    }

    fn count_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = (x as i32 + dx).rem_euclid(self.width as i32) as usize;
                let ny = (y as i32 + dy).rem_euclid(self.height as i32) as usize;

                if self.grid[ny][nx] {
                    count += 1;
                }
            }
        }
        count
    }

    fn toggle_cell(&mut self, x: usize, y: usize) {
        if y < self.height && x < self.width {
            self.grid[y][x] = !self.grid[y][x];
        }
    }

    fn toggle_running(&mut self) {
        self.running = !self.running;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(50, 30);
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| {
            let size = f.size();
            
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Game of Life (Space: Play/Pause, Enter: Step, q: Quit)");
            
            let mut cells = String::new();
            for row in &app.grid {
                for &cell in row {
                    cells.push(if cell { '█' } else { ' ' });
                }
                cells.push('\n');
            }
            
            let paragraph = Paragraph::new(cells)
                .style(Style::default().fg(Color::White))
                .block(block);
            
            f.render_widget(paragraph, size);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => app.toggle_running(),
                    KeyCode::Enter => {
                        if !app.running {
                            app.update();
                        }
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if app.running {
                app.update();
            }
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

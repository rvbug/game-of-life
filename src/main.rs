//! # Game of Life
//! 
//! A terminal-based implementation of Conway's Game of Life using the `ratatui` library
//! for terminal user interface and `crossterm` for terminal manipulation.
//! 
//! ## Features
//! 
//! * Interactive terminal interface
//! * Real-time statistics tracking
//! * System resource monitoring
//! * Configurable grid size
//! * Toroidal grid implementation

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::{
    error::Error, 
    io,
    time::{Duration, Instant},
    process,
};
use rand::Rng;
use sysinfo::{System, SystemExt};


/// Stores statistics about the Game of Life simulation.
#[derive(Debug)]
struct Stats {
    /// Current generation number
    generation: u64,
    /// Total number of cells created since start
    cells_created: u64,
    /// Total number of cells destroyed since start
    cells_destroyed: u64,
    /// Current number of living cells
    current_population: u64,
}

impl Stats {
    /// Creates a new `Stats` instance with all counters initialized to zero.
    fn new() -> Self {
        Stats {
            generation: 0,
            cells_created: 0,
            cells_destroyed: 0,
            current_population: 0,
        }
    }
}

/// Main application state container for the Game of Life simulation.
#[derive(Debug)] 
struct App {
    /// The game board represented as a 2D vector of booleans where true indicates a live cell
    grid: Vec<Vec<bool>>,
    /// Width of the game board
    width: usize,
    /// Height of the game board
    height: usize,
    /// Indicates whether the simulation is currently running
    running: bool,
    /// Statistics tracking for the simulation
    stats: Stats,
    /// System information for resource monitoring
    sys: System,
}

impl App {
    /// Creates a new Game of Life application with the specified dimensions.
    /// 
    /// # Arguments
    /// 
    /// * `width` - The width of the game board
    /// * `height` - The height of the game board
    /// 
    /// # Returns
    /// 
    /// A new `App` instance with a randomly initialized grid where approximately 30%
    /// of cells are alive.

    
    fn new(width: usize, height: usize) -> App {
        let mut rng = rand::thread_rng();
        let grid = (0..height)
            .map(|_| (0..width).map(|_| rng.gen_bool(0.3)).collect())
            .collect();

        let mut app = App {
            grid,
            width,
            height,
            running: false,
            stats: Stats::new(),
            sys: System::new_all(),
        };
        
        // Calculate initial population
        app.stats.current_population = app.count_total_alive();
        app
    }
    
    /// Counts the total number of living cells in the grid.
    /// 
    /// # Returns
    /// 
    /// The number of cells that are currently alive.
    fn count_total_alive(&self) -> u64 {
        self.grid.iter()
            .flat_map(|row| row.iter())
            .filter(|&&cell| cell)
            .count() as u64
    }
    
    /// Updates the grid to the next generation according to Conway's Game of Life rules:
    /// 
    /// * Any live cell with fewer than two live neighbors dies (underpopulation)
    /// * Any live cell with two or three live neighbors survives
    /// * Any live cell with more than three live neighbors dies (overpopulation)
    /// * Any dead cell with exactly three live neighbors becomes alive (reproduction)
    fn update(&mut self) {
        let mut new_grid = self.grid.clone();
        let mut cells_created = 0;
        let mut cells_destroyed = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let live_neighbors = self.count_neighbors(x, y);
                let cell = self.grid[y][x];
                let new_state = match (cell, live_neighbors) {
                    (true, x) if x < 2 => {
                        cells_destroyed += 1;
                        false
                    },
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => {
                        cells_destroyed += 1;
                        false
                    },
                    (false, 3) => {
                        cells_created += 1;
                        true
                    },
                    (otherwise, _) => otherwise,
                };
                new_grid[y][x] = new_state;
            }
        }

        self.grid = new_grid;
        self.stats.generation += 1;
        self.stats.cells_created += cells_created;
        self.stats.cells_destroyed += cells_destroyed;
        self.stats.current_population = self.count_total_alive();
        self.sys.refresh_memory();
    }

    /// Counts the number of live neighbors for a cell at the specified coordinates.
    /// 
    /// The grid is treated as toroidal, meaning the edges wrap around to the opposite side.
    /// 
    /// # Arguments
    /// 
    /// * `x` - The x-coordinate of the cell
    /// * `y` - The y-coordinate of the cell
    /// 
    /// # Returns
    /// 
    /// The number of live neighbors (0-8)
    
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
    
    /// Toggles the simulation between running and paused states.
    fn toggle_running(&mut self) {
        self.running = !self.running;
    }
}

/// Draws the game grid to the terminal interface.
/// 
/// # Arguments
/// 
/// * `f` - The frame to draw on
/// * `app` - The application state
/// * `area` - The area of the terminal to draw in
fn draw_grid(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Game of Life [Space: Play/Pause | Enter: Step | q: Quit]");
    
    let mut cells = String::new();
    for row in &app.grid {
        for &cell in row {
            cells.push(if cell { '•' } else { ' ' });
        }
        cells.push('\n');
    }
    
    let paragraph = Paragraph::new(cells)
        .style(Style::default().fg(Color::White))
        .block(block);
    
    f.render_widget(paragraph, area);
}

/// Draws the statistics panel to the terminal interface.
/// 
/// # Arguments
/// 
/// * `f` - The frame to draw on
/// * `app` - The application state
/// * `area` - The area of the terminal to draw in
fn draw_stats(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let memory_used = app.sys.used_memory() / 1024; // Convert to KB
    let memory_total = app.sys.total_memory() / 1024;
    
    let stats_text = format!(
        "Statistics:\n\
        Generation: {}\n\
        Current Population: {}\n\
        Cells Created: {}\n\
        Cells Destroyed: {}\n\
        Birth Rate: {:.2}/gen\n\
        Death Rate: {:.2}/gen\n\
        Memory Usage: {}KB/{:.2}MB\n\
        Status: {}\n",
        app.stats.generation,
        app.stats.current_population,
        app.stats.cells_created,
        app.stats.cells_destroyed,
        app.stats.cells_created as f64 / app.stats.generation.max(1) as f64,
        app.stats.cells_destroyed as f64 / app.stats.generation.max(1) as f64,
        memory_used,
        memory_total as f64 / 1024.0,
        if app.running { "Running" } else { "Paused" }
    );

    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Statistics"))
        .wrap(Wrap { trim: true });

    f.render_widget(stats_widget, area);
}

/// Main entry point for the Game of Life application.
/// 
/// Sets up the terminal interface, creates the initial game state, and runs the main event loop.
/// The game runs at 10 FPS (100ms intervals) when active.
/// 
/// # Controls
/// 
/// * Space: Play/Pause the simulation
/// * Enter: Step forward one generation (when paused)
/// * q: Quit the application
/// 
/// # Errors
/// 
/// Returns an error if terminal manipulation fails.
fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(80, 40);
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(75),
                    Constraint::Percentage(25),
                ].as_ref())
                .split(f.size());
            
            draw_grid(f, &app, chunks[0]);
            draw_stats(f, &app, chunks[1]);
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

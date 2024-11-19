# Game of Life
To implement game of life in rust


# Game of Life (v 1.0)

A terminal-based implementation of Conway's Game of Life written in Rust using the `ratatui` library for the terminal user interface and `crossterm` for terminal manipulation.

## Overview

This program implements Conway's Game of Life with an interactive terminal interface, featuring:
- Real-time visualization of the cellular automaton
- Statistics tracking
- System resource monitoring
- Interactive controls

## Dependencies

```toml
[dependencies]
crossterm = "..."      # Terminal manipulation and event handling
ratatui = "..."        # Terminal user interface framework
rand = "..."           # Random number generation
sysinfo = "..."        # System information monitoring
```

## Core Components

### Stats Structure

```rust
struct Stats {
    generation: u64,        // Current generation number
    cells_created: u64,     // Total cells created
    cells_destroyed: u64,   // Total cells destroyed
    current_population: u64, // Current living cells count
}
```

The `Stats` structure maintains runtime statistics about the simulation.

### App Structure

```rust
struct App {
    grid: Vec<Vec<bool>>,   // The game board
    width: usize,           // Grid width
    height: usize,          // Grid height
    running: bool,          // Simulation state
    stats: Stats,           // Runtime statistics
    sys: System,           // System resource monitoring
}
```

The `App` structure is the main container for the game state and logic.

## Key Features

### Grid Management
- Implements a toroidal grid where edges wrap around
- Uses boolean values for cell states (true = alive, false = dead)
- Initial state is randomly generated with 30% probability of live cells

### Game Rules
The classic Conway's Game of Life rules are implemented:
1. Any live cell with fewer than two live neighbors dies (underpopulation)
2. Any live cell with two or three live neighbors survives
3. Any live cell with more than three live neighbors dies (overpopulation)
4. Any dead cell with exactly three live neighbors becomes alive (reproduction)

### Statistics Tracking
- Tracks generations
- Monitors cell creation and destruction
- Calculates birth and death rates
- Displays current population
- Shows system memory usage

## User Interface

### Controls
- `Space`: Toggle simulation play/pause
- `Enter`: Step forward one generation (when paused)
- `q`: Quit the application

### Display
The interface is split into two main sections:
1. Game Grid (75% of width)
   - Displays the current state of cells
   - Live cells shown as '•'
   - Dead cells shown as spaces
   - Bordered with title and controls

2. Statistics Panel (25% of width)
   - Shows real-time statistics
   - Displays system resource usage
   - Indicates simulation status

## Implementation Details

### Grid Updates
The `update()` method implements the core Game of Life logic:
- Creates a new grid for the next generation
- Applies rules to each cell based on neighbor count
- Updates statistics
- Refreshes system memory information

### Neighbor Counting
The `count_neighbors()` method:
- Implements toroidal wrapping using modulo arithmetic
- Counts live neighbors in all 8 adjacent cells
- Handles edge cases correctly

### Performance Considerations
- Uses efficient boolean grid representation
- Updates occur at fixed intervals (100ms by default)
- Maintains separate update and render loops
- Uses event polling to remain responsive

## Error Handling
- Implements proper terminal cleanup on exit
- Uses Result type for error propagation
- Handles terminal mode changes safely

## Usage Example

```rust
fn main() -> Result<(), Box<dyn Error>> {
    // Initialize app with 80x40 grid
    let mut app = App::new(80, 40);
    
    // Main loop handles:
    // - Drawing the interface
    // - Processing user input
    // - Updating the simulation
    // - Managing terminal state
}
```

## Future Improvements
Potential areas for enhancement:
1. Add save/load functionality for grid states
2. Implement different initial patterns
3. Add color support for different cell ages
4. Add configuration options for update speed
5. Implement mouse support for cell toggling
6. Add different visualization modes

## Notes
- The grid wraps around at the edges (toroidal)
- The simulation runs at 10 FPS (100ms intervals)
- Initial population density is set to 30%
- Memory usage is displayed in KB/MB


# Future 
2nd version will have tcp server running multiple clients

# Xonix

A Rust implementation of the classic Xonix territory-capturing game with a CLI interface.

## Game Overview

Xonix is a territory-capturing game where you control a cursor that moves around the playfield. The objective is to claim territory by drawing lines into empty space and returning to filled areas. Watch out for bouncing balls that can destroy your trail!

### Rules
- Move your cursor with arrow keys
- Draw trails from filled areas into empty space
- Return to filled areas to capture territory
- Avoid balls bouncing in empty areas
- Don't cross your own trail before reaching safety
- Fill 75% of the field to win and advance to the next level

## Architecture

The project is structured to separate game logic from rendering, making it easy to add new rendering backends (like WebAssembly/Canvas):

```
src/
├── lib.rs              # Library exports
├── entity.rs           # Game entities (Player, Ball, Position, Direction)
├── game.rs             # Core game logic and state management
├── renderer.rs         # Renderer trait (abstraction layer)
├── cli_renderer.rs     # CLI implementation using crossterm
└── main.rs             # Game loop for CLI binary
```

### Key Design Decisions

1. **Renderer Trait**: The `Renderer` trait abstracts all rendering operations, allowing different backends to be implemented without changing game logic.

2. **Separation of Concerns**:
   - `game.rs` contains pure game logic (no rendering)
   - `cli_renderer.rs` handles all CLI-specific rendering
   - `main.rs` orchestrates the game loop

3. **WASM-Ready**: The architecture makes it straightforward to add a web renderer:
   - Implement the `Renderer` trait for Canvas/WebGL
   - Create a new binary target for WASM compilation
   - Reuse all game logic from the library

## Building and Running

### Prerequisites
- Rust 1.70 or later

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run --bin xonix-cli
```

## Controls

- **Arrow Keys**: Move the cursor
- **Q**: Quit game
- **R**: Restart game
- **Space**: Advance to next level (when level is won)

## Adding a Web Renderer

To add a web renderer in the future:

1. Create a new module `src/web_renderer.rs`
2. Implement the `Renderer` trait for your web rendering backend
3. Create a new binary `src/web_main.rs` with WASM bindings
4. Add WASM build target to `Cargo.toml`

Example structure:
```rust
// src/web_renderer.rs
pub struct WebRenderer {
    canvas: web_sys::HtmlCanvasElement,
    context: web_sys::CanvasRenderingContext2d,
}

impl Renderer for WebRenderer {
    // Implement rendering using Canvas API
}
```

## License

MIT

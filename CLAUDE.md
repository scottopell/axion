# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Axion is a Rust implementation of the classic territory-capturing game with both CLI and Web (WASM) interfaces. The game features smooth 60 FPS animations in the web version while maintaining consistent 10 Hz gameplay logic.

## Build Commands

### CLI Version
```bash
# Build
cargo build --release

# Run
cargo run --bin axion-cli

# Run optimized binary
./target/release/axion-cli
```

### Web Version (WASM)
```bash
# Build (requires wasm-pack: cargo install wasm-pack)
./build-web.sh
# Or manually:
wasm-pack build --target web --out-dir www/pkg

# Run local server
cd www
python3 -m http.server 8080
# Then open http://localhost:8080
```

### Testing
```bash
# Run all tests
cargo test

# Run property-based tests (uses proptest)
cargo test --release

# Run specific test module
cargo test game::tests
cargo test ball_trail::tests
```

## Architecture

### Core Design: Renderer Trait Pattern

The codebase uses a **renderer trait abstraction** to separate game logic from rendering implementation. This allows the same game logic to power both CLI and Web versions.

**Key principle:** `game.rs` contains pure game logic with zero rendering code. All display code lives in renderer implementations.

### Module Structure

- **`game.rs`** - Core game state and logic
  - Game loop-independent (no timing/rendering)
  - Territory capture via flood-fill algorithm (lines 196-313)
  - Win condition: 75% territory filled
  - Ball collision detection and bouncing physics

- **`entity.rs`** - Game entities (Player, Ball, Position, Direction)
  - Player has trail tracking for territory capture
  - Enemy trait allows for extensible enemy types

- **`renderer.rs`** - Trait definition for rendering backends
  - `init()`, `render()`, `cleanup()`, `poll_input()`
  - Abstraction allows different frame rates from game logic

- **`cli_renderer.rs`** - Terminal rendering using crossterm (CLI only)
- **`web_renderer.rs`** - Canvas 2D rendering (WASM only)
- **`ball_trail.rs`** - Motion blur trail system with discontinuity detection

- **`main.rs`** - CLI game loop (10 Hz updates)
- **`web_main.rs`** - WASM entry point with 60 FPS rendering

### Platform-Specific Code

The codebase uses conditional compilation for platform-specific code:

```rust
// CLI dependencies (native only)
#[cfg(not(target_arch = "wasm32"))]

// WASM dependencies
#[cfg(target_arch = "wasm32")]
```

**Important:** Renderer implementations are conditionally compiled. `CliRenderer` only exists on native targets; `WebRenderer` only on WASM.

### Critical Gameplay Logic

**Territory Capture Algorithm** (game.rs:196-313):
1. When player completes a trail (returns to filled area), trigger flood fill
2. Find ALL empty regions using BFS
3. Identify the LARGEST empty region (the "outside" playable area)
4. All smaller regions are considered "enclosed"
5. Fill regions without balls (safe to capture)
6. If all enclosed regions have balls, fill the smallest one

This "largest region = outside" approach is the classic territory-capture behavior (inspired by Xonix).

**Movement Constraints:**
- Cannot reverse direction while drawing a trail (game.rs:77-84)
- Can reverse freely on safe (filled) territory
- Moving into own trail = instant loss

**Ball Physics:**
- Balls bounce off walls and filled cells (game.rs:136-145)
- Move 1 cell per game tick (10 Hz)
- Ball hitting player or trail = instant loss

### Animation System (Web Only)

The web version achieves smooth 60 FPS via **position interpolation**:

1. Game logic updates at 10 Hz (every 100ms)
2. Renderer updates at 60 FPS (~16ms)
3. Between game ticks, entity positions are interpolated using lerp
4. Visual result: smooth gliding motion despite discrete game updates

**BallTrail System** (ball_trail.rs):
- Tracks last 6 interpolated positions for motion blur
- Automatically clears on discontinuities (>1 cell jump, indicates bounce)
- Distance validation ensures trail stays near ball

## Testing Strategy

The codebase uses **property-based testing** via proptest to catch edge cases:

**Key Invariants Tested:**
- Filled percentage never exceeds 100%
- Fill percentage monotonically increases (never decreases)
- Small trails cannot cause massive fills
- Balls always stay within bounds
- Borders always remain filled
- Win condition requires target percentage

**Note:** Some property tests are marked `#[ignore]` due to flawed test design (not code bugs).

## Common Development Patterns

### Running a Single Test
```bash
# Run specific test
cargo test test_adjacent_trail_to_border_doesnt_fill_entire_board

# Run with output
cargo test test_simple_corner_enclosure -- --nocapture
```

### Adding New Enemies
1. Implement the `Enemy` trait (entity.rs:77-80)
2. Add enemy type to game state
3. Update game.rs collision detection
4. Add rendering in both CLI and Web renderers

### Modifying Game Mechanics
- **Speed:** Adjust `GAME_UPDATE_RATE` in main.rs (CLI) or web_main.rs (Web)
- **Win condition:** Modify `target_percentage` in game.rs:28
- **Board size:** Change constants in main.rs (CLI) or web initialization (Web)

## File Locations

- Source: `src/`
- Web assets: `www/` (HTML, CSS, JS, and generated `pkg/` from wasm-pack)
- Proptest regression data: `proptest-regressions/`
- Build script: `build-web.sh`

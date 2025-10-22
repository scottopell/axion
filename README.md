# Axion

A Rust implementation of the classic territory-capturing game with CLI and Web (WASM) interfaces.

## Game Rules

Control a cursor to claim territory by drawing lines from filled areas into empty space. Return to safety to capture territory. Avoid bouncing balls and your own trail. Fill 75% to win.

**Controls:** Arrow keys (move), Q (quit), R (restart), Space (next level)

## Quick Start

### CLI Version
```bash
cargo build --release
cargo run --bin axion-cli
```

### Web Version (60 FPS)
```bash
./build-web.sh          # Requires: cargo install wasm-pack
cd docs
python3 -m http.server 8080
# Open http://localhost:8080
```

**Web Features:** 60 FPS interpolated movement, ball motion blur trails, animated territory capture, retro pixel art aesthetic with CRT effects.

## Architecture

Renderer trait abstraction separates game logic from display. Game logic runs at 10 Hz; web renderer displays at 60 FPS via position interpolation (lerp).

```
src/
├── game.rs             # Core game logic (platform-agnostic)
├── entity.rs           # Game entities (Player, Ball, Direction)
├── renderer.rs         # Renderer trait abstraction
├── cli_renderer.rs     # Terminal rendering (crossterm)
├── web_renderer.rs     # Canvas 2D rendering (WASM)
├── ball_trail.rs       # Motion blur trail system
├── main.rs             # CLI entry point
└── web_main.rs         # WASM entry point
```

**Key Design:** `game.rs` contains zero rendering code. All display logic lives in renderer implementations, allowing the same game logic to power both CLI and web versions.

### Interpolation Example

Game updates entity positions discretely at 10 Hz. Between updates, the web renderer smoothly interpolates positions at 60 FPS:

```
Ball moves (5,5) → (6,5):
Frame 1: 5.0    Frame 4: 5.5    Frame 7: 6.0
Frame 2: 5.17   Frame 5: 5.67
Frame 3: 5.33   Frame 6: 5.83
```

Result: Buttery-smooth gliding motion despite discrete game updates.

### Territory Capture Algorithm

Classic flood-fill logic (game.rs:196-313):
1. Player completes trail → trigger flood fill
2. Find all empty regions via BFS
3. Largest region = "outside" playable area
4. Fill smaller enclosed regions (prioritize ball-free regions)

## Testing

```bash
cargo test                           # Run all tests
cargo test game::tests               # Specific module
cargo test --release                 # Property tests (proptest)
```

Property-based tests validate invariants: fill percentage ≤100%, monotonic increase, balls stay in bounds, borders stay filled.

## File Structure

```
.
├── src/                # Rust source code
├── docs/               # Web assets (HTML, CSS, JS)
│   └── pkg/            # Generated WASM files (for GitHub Pages)
├── build-web.sh        # Web build script
├── Cargo.toml          # Platform-conditional dependencies
├── CLAUDE.md           # Development instructions for AI
└── proptest-regressions/  # Test regression data (gitignored)
```

## Comparison to Original Xonix

Axion is a faithful modernization of the 1984 DOS game Xonix (itself inspired by the arcade game QIX). The core territory-capture mechanics are preserved while making targeted simplifications for accessibility.

### What's Preserved
- ✅ Territory capture via trail drawing and flood fill
- ✅ 75% fill threshold to win
- ✅ Cannot reverse direction while drawing trail
- ✅ Balls bounce off walls and filled territory
- ✅ Ball collision with player/trail = instant loss
- ✅ Regions containing balls are NOT auto-filled (strategic core)
- ✅ Level progression with increasing ball count

### Potential Future Enhancements
The original Xonix/QIX included additional mechanics that could be added:

**Edge-Patrolling Enemies (Sparx):**
- Enemies that travel along filled territory edges
- "Super Sparx" variants that can chase player along unfinished trails
- Would add pressure to complete trails quickly

**Fast/Slow Draw Speed:**
- Hold modifier key for "slow draw" mode (double points, slower movement)
- Default "fast draw" for normal points and speed
- Adds risk/reward scoring strategy

**Idle Fuse Mechanic:**
- If player stops while drawing, a "fuse" burns along the trail toward them
- Forces continuous movement while exposed
- Death if fuse catches the player

### Design Philosophy
The current implementation focuses on the core loop: carefully carving territory while avoiding balls. The simplified enemy behavior (no edge-patrollers, no active pursuit) makes the game more approachable while preserving strategic depth. The player-position-based flood fill algorithm improves on the original's size-only approach.

## License

MIT

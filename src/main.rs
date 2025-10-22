use crossterm::terminal;
use std::io;
use std::time::{Duration, Instant};
use axion::{CliRenderer, Game, GameState, Input, Renderer};

// Game logic update rate (controls gameplay speed)
const GAME_UPDATE_RATE: Duration = Duration::from_millis(100); // 10 updates/sec

fn main() -> io::Result<()> {
    // Get terminal size and calculate game dimensions
    let (term_width, term_height) = terminal::size()?;

    // Account for:
    // - Each cell is 2 chars wide, so width = term_width / 2
    // - Reserve 4 lines at bottom for info display
    // - Minimum size of 20x10 for playability
    let game_width = ((term_width / 2) as i32).max(20);
    let game_height = ((term_height - 4) as i32).max(10);

    let mut game = Game::new(game_width, game_height);
    let mut renderer = CliRenderer::new();

    renderer.init()?;

    let mut last_game_update = Instant::now();

    loop {
        // Poll for input
        if let Some(input) = renderer.poll_input()? {
            match input {
                Input::Direction(direction) => {
                    game.set_direction(direction);
                }
                Input::Quit => {
                    break;
                }
                Input::Restart => {
                    game.reset();
                }
                Input::NextLevel if game.state == GameState::Won => {
                    game.next_level();
                }
                _ => {}
            }
        }

        // Update game logic at fixed rate
        if last_game_update.elapsed() >= GAME_UPDATE_RATE {
            game.update();
            last_game_update = Instant::now();
        }

        // Let renderer decide when to actually render
        // (it manages its own frame rate internally)
        renderer.render(&game)?;
    }

    renderer.cleanup()?;
    Ok(())
}

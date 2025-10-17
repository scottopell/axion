use std::io;
use std::time::{Duration, Instant};
use xonix::{CliRenderer, Game, GameState, Input, Renderer};

const GAME_WIDTH: i32 = 40;
const GAME_HEIGHT: i32 = 20;

// Game logic update rate (controls gameplay speed)
const GAME_UPDATE_RATE: Duration = Duration::from_millis(100); // 10 updates/sec

fn main() -> io::Result<()> {
    let mut game = Game::new(GAME_WIDTH, GAME_HEIGHT);
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

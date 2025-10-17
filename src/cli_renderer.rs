use crate::entity::Direction;
use crate::game::{Cell, Game, GameState};
use crate::renderer::{Input, Renderer};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::time::{Duration, Instant};

pub struct CliRenderer {
    last_render: Instant,
    target_frame_time: Duration,
}

impl CliRenderer {
    pub fn new() -> Self {
        Self {
            last_render: Instant::now(),
            // Target 30 FPS for smooth rendering
            target_frame_time: Duration::from_millis(33),
        }
    }

    fn draw_cell(&self, cell: Cell, stdout: &mut io::Stdout) -> io::Result<()> {
        match cell {
            Cell::Empty => {
                queue!(stdout, SetBackgroundColor(Color::Black), Print("  "))?;
            }
            Cell::Filled => {
                queue!(stdout, SetBackgroundColor(Color::Blue), Print("  "))?;
            }
            Cell::Trail => {
                queue!(stdout, SetBackgroundColor(Color::Yellow), Print("  "))?;
            }
        }
        Ok(())
    }

    fn draw_info(&self, game: &Game, stdout: &mut io::Stdout) -> io::Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(0, (game.height + 1) as u16),
            ResetColor,
            Print(format!(
                "Level: {}  Score: {}  Filled: {:.1}%  Target: {:.0}%",
                game.level,
                game.score,
                game.filled_percentage * 100.0,
                game.target_percentage * 100.0
            ))
        )?;

        queue!(
            stdout,
            cursor::MoveTo(0, (game.height + 2) as u16),
            Print("Controls: Arrow Keys to move | Q to quit | R to restart")
        )?;

        match game.state {
            GameState::Won => {
                queue!(
                    stdout,
                    cursor::MoveTo(0, (game.height + 3) as u16),
                    SetForegroundColor(Color::Green),
                    Print("YOU WIN! Press SPACE for next level or R to restart"),
                    ResetColor
                )?;
            }
            GameState::Lost => {
                queue!(
                    stdout,
                    cursor::MoveTo(0, (game.height + 3) as u16),
                    SetForegroundColor(Color::Red),
                    Print("GAME OVER! Press R to restart"),
                    ResetColor
                )?;
            }
            GameState::Playing => {}
        }

        Ok(())
    }
}

impl Renderer for CliRenderer {
    fn init(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            terminal::Clear(ClearType::All),
            cursor::Hide
        )?;
        Ok(())
    }

    fn render(&mut self, game: &Game) -> io::Result<()> {
        // Frame rate limiting: skip rendering if not enough time has passed
        if self.last_render.elapsed() < self.target_frame_time {
            return Ok(());
        }

        self.last_render = Instant::now();

        let mut stdout = io::stdout();

        queue!(stdout, cursor::MoveTo(0, 0))?;

        // Draw board
        for y in 0..game.height {
            for x in 0..game.width {
                let cell = game.cell_at(x, y);

                // Check if this is the player position
                if game.player.position.x == x && game.player.position.y == y {
                    queue!(
                        stdout,
                        SetBackgroundColor(Color::Green),
                        SetForegroundColor(Color::Black),
                        Print("@@")
                    )?;
                    continue;
                }

                // Check if this is a ball position
                let mut is_ball = false;
                for ball in &game.balls {
                    if ball.position.x == x && ball.position.y == y {
                        queue!(
                            stdout,
                            SetBackgroundColor(Color::Black),
                            SetForegroundColor(Color::Red),
                            Print("()"),
                            ResetColor
                        )?;
                        is_ball = true;
                        break;
                    }
                }

                if !is_ball {
                    self.draw_cell(cell, &mut stdout)?;
                }
            }
            queue!(stdout, ResetColor, Print("\r\n"))?;
        }

        // Draw info
        self.draw_info(game, &mut stdout)?;

        stdout.flush()?;
        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            cursor::Show,
            terminal::LeaveAlternateScreen,
            ResetColor
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn poll_input(&mut self) -> io::Result<Option<Input>> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        return Ok(Some(Input::Quit));
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        return Ok(Some(Input::Restart));
                    }
                    KeyCode::Char(' ') => {
                        return Ok(Some(Input::NextLevel));
                    }
                    KeyCode::Up => return Ok(Some(Input::Direction(Direction::Up))),
                    KeyCode::Down => return Ok(Some(Input::Direction(Direction::Down))),
                    KeyCode::Left => return Ok(Some(Input::Direction(Direction::Left))),
                    KeyCode::Right => return Ok(Some(Input::Direction(Direction::Right))),
                    _ => {}
                }
            }
        }
        Ok(None)
    }
}

impl Drop for CliRenderer {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

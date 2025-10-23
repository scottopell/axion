use crate::game::Game;
use crate::entity::Direction;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Direction(Direction),
    Quit,
    Restart,
    NextLevel,
    Tap, // Mobile tap gesture - handled contextually based on game state
}

/// Trait that abstracts rendering implementation.
/// This allows for different rendering backends (CLI, Web, etc.)
pub trait Renderer {
    /// Initialize the renderer
    fn init(&mut self) -> io::Result<()>;

    /// Render the current game state
    fn render(&mut self, game: &Game) -> io::Result<()>;

    /// Clean up and restore terminal/display state
    fn cleanup(&mut self) -> io::Result<()>;

    /// Poll for input from the user
    fn poll_input(&mut self) -> io::Result<Option<Input>>;
}

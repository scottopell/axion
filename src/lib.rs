pub mod game;
pub mod entity;
pub mod renderer;
pub mod ball_trail;

#[cfg(not(target_arch = "wasm32"))]
pub mod cli_renderer;

#[cfg(target_arch = "wasm32")]
pub mod web_renderer;

#[cfg(target_arch = "wasm32")]
pub mod web_main;

pub use game::{Game, GameState, Cell};
pub use entity::{Position, Direction, Player, Ball, Enemy};
pub use renderer::{Renderer, Input};

#[cfg(not(target_arch = "wasm32"))]
pub use cli_renderer::CliRenderer;

#[cfg(target_arch = "wasm32")]
pub use web_renderer::WebRenderer;

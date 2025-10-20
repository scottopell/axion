use crate::{Game, GameState, Input, Renderer, WebRenderer};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const GAME_WIDTH: i32 = 40;
const GAME_HEIGHT: i32 = 20;
const GAME_UPDATE_INTERVAL: f64 = 100.0; // 10 Hz game logic

struct GameLoop {
    game: Game,
    renderer: WebRenderer,
    last_update: f64,
}

impl GameLoop {
    fn new() -> Result<Self, JsValue> {
        let game = Game::new(GAME_WIDTH, GAME_HEIGHT);
        let mut renderer = WebRenderer::new("gameCanvas")?;
        renderer.init().map_err(|e| JsValue::from_str(&e.to_string()))?;

        let window = web_sys::window().ok_or("no window")?;
        let performance = window.performance().ok_or("no performance")?;
        let last_update = performance.now();

        Ok(Self {
            game,
            renderer,
            last_update,
        })
    }

    fn update_frame(&mut self, current_time: f64) -> Result<(), JsValue> {
        // Poll for input
        if let Some(input) = self
            .renderer
            .poll_input()
            .map_err(|e| JsValue::from_str(&e.to_string()))?
        {
            match input {
                Input::Direction(direction) => {
                    self.game.set_direction(direction);
                }
                Input::Quit => {
                    web_sys::console::log_1(&"Game quit".into());
                    // In web, we can't really quit, just log it
                }
                Input::Restart => {
                    self.game.reset();
                }
                Input::NextLevel if self.game.state == GameState::Won => {
                    self.game.next_level();
                }
                _ => {}
            }
        }

        // Update game logic at fixed rate
        if current_time - self.last_update >= GAME_UPDATE_INTERVAL {
            self.game.update();
            self.last_update = current_time;
        }

        // Render (renderer manages its own frame rate)
        self.renderer
            .render(&self.game)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(())
    }
}

#[wasm_bindgen]
pub fn start_game() -> Result<(), JsValue> {
    // Set panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"[WASM] Starting Xonix initialization...".into());

    // Create game loop
    web_sys::console::log_1(&"[WASM] Creating game loop...".into());
    let game_loop = match GameLoop::new() {
        Ok(gl) => {
            web_sys::console::log_1(&"[WASM] Game loop created successfully!".into());
            Rc::new(RefCell::new(gl))
        }
        Err(e) => {
            web_sys::console::error_1(&format!("[WASM] Failed to create game loop: {:?}", e).into());
            return Err(e);
        }
    };

    // Setup requestAnimationFrame loop
    web_sys::console::log_1(&"[WASM] Setting up animation loop...".into());
    let window = web_sys::window().ok_or("no window")?;
    let performance = window.performance().ok_or("no performance")?;

    // Create closure for animation frame
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let game_loop_clone = game_loop.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let current_time = performance.now();

        // Update game frame
        if let Err(e) = game_loop_clone.borrow_mut().update_frame(current_time) {
            web_sys::console::error_1(&e);
            return; // Stop loop on error
        }

        // Schedule next frame
        let window = web_sys::window().unwrap();
        window
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    // Start the loop
    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();

    web_sys::console::log_1(&"[WASM] Game loop started! Game should now be running.".into());

    Ok(())
}

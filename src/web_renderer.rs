use crate::ball_trail::BallTrail;
use crate::entity::{Direction, Position};
use crate::game::{Cell, Game, GameState};
use crate::renderer::{Input, Renderer};
use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, KeyboardEvent, TouchEvent};

const CELL_SIZE: f64 = 16.0;
const TARGET_FRAME_TIME: f64 = 16.0; // ~60 FPS
const GAME_UPDATE_INTERVAL: f64 = 100.0; // Game logic updates at 10 Hz
const SWIPE_THRESHOLD: f64 = 30.0; // Minimum distance in pixels to register a swipe

// Colors (retro palette)
const COLOR_EMPTY: &str = "#000000";
const COLOR_FILLED: &str = "#0000AA";
const COLOR_TRAIL: &str = "#FFFF55";
const COLOR_PLAYER: &str = "#55FF55";
const COLOR_BALL: &str = "#FF5555";
const COLOR_UI: &str = "#FFFFFF";

/// Snapshot of game state for interpolation
#[derive(Clone)]
struct GameSnapshot {
    player_pos: Position,
    ball_positions: Vec<Position>,
    board_hash: u64, // Simple hash to detect board changes
}

impl GameSnapshot {
    fn from_game(game: &Game) -> Self {
        Self {
            player_pos: game.player.position,
            ball_positions: game.balls.iter().map(|b| b.position).collect(),
            board_hash: Self::hash_board(&game.board),
        }
    }

    fn hash_board(board: &[Vec<Cell>]) -> u64 {
        // Simple hash: count filled cells
        let mut count = 0u64;
        for row in board {
            for cell in row {
                if matches!(cell, Cell::Filled) {
                    count += 1;
                }
            }
        }
        count
    }
}

// BallTrail moved to ball_trail.rs module for testing

/// Fill flood animation state
struct FloodFillAnimation {
    cells_to_animate: Vec<(i32, i32)>,
    animation_start: f64,
    duration: f64,
}

impl FloodFillAnimation {
    fn new(cells: Vec<(i32, i32)>, start_time: f64) -> Self {
        Self {
            cells_to_animate: cells,
            animation_start: start_time,
            duration: 400.0, // 400ms animation
        }
    }

    fn progress(&self, current_time: f64) -> f64 {
        let elapsed = current_time - self.animation_start;
        (elapsed / self.duration).min(1.0)
    }

    fn is_complete(&self, current_time: f64) -> bool {
        self.progress(current_time) >= 1.0
    }
}

pub struct WebRenderer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    window: web_sys::Window,
    device_pixel_ratio: f64,

    // Interpolation state
    last_game_snapshot: Option<GameSnapshot>,
    last_update_time: f64,
    last_render_time: f64,

    // Visual effects
    ball_trails: Vec<BallTrail>,
    fill_animation: Option<FloodFillAnimation>,

    // Input state
    pending_input: Rc<RefCell<Option<Input>>>,

    // Touch state
    touch_start_pos: Rc<RefCell<Option<(f64, f64)>>>,
}

impl WebRenderer {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("canvas not found")?
            .dyn_into::<HtmlCanvasElement>()?;

        let context = canvas
            .get_context("2d")?
            .ok_or("no 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        // Disable image smoothing for crisp pixels
        context.set_image_smoothing_enabled(false);

        // Get device pixel ratio for high DPI displays
        let device_pixel_ratio = window.device_pixel_ratio();

        let pending_input = Rc::new(RefCell::new(None));
        let touch_start_pos = Rc::new(RefCell::new(None));

        Ok(Self {
            canvas,
            context,
            window,
            device_pixel_ratio,
            last_game_snapshot: None,
            last_update_time: 0.0,
            last_render_time: 0.0,
            ball_trails: Vec::new(),
            fill_animation: None,
            pending_input,
            touch_start_pos,
        })
    }

    fn setup_keyboard_listener(&self) {
        let pending_input = self.pending_input.clone();

        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let input = match event.key().as_str() {
                "ArrowUp" => Some(Input::Direction(Direction::Up)),
                "ArrowDown" => Some(Input::Direction(Direction::Down)),
                "ArrowLeft" => Some(Input::Direction(Direction::Left)),
                "ArrowRight" => Some(Input::Direction(Direction::Right)),
                "q" | "Q" => Some(Input::Quit),
                "r" | "R" => Some(Input::Restart),
                " " => Some(Input::NextLevel),
                _ => None,
            };

            if let Some(input) = input {
                *pending_input.borrow_mut() = Some(input);
                event.prevent_default();
            }
        }) as Box<dyn FnMut(KeyboardEvent)>);

        self.window
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget(); // Keep listener alive
    }

    fn setup_touch_listeners(&self) {
        let pending_input = self.pending_input.clone();
        let touch_start_pos = self.touch_start_pos.clone();
        let canvas = self.canvas.clone();

        // TouchStart: Record initial position
        let touch_start_pos_clone = touch_start_pos.clone();
        let touchstart_closure = Closure::wrap(Box::new(move |event: TouchEvent| {
            event.prevent_default(); // Prevent zooming, scrolling, etc.

            if let Some(touch) = event.touches().item(0) {
                let x = touch.client_x() as f64;
                let y = touch.client_y() as f64;
                *touch_start_pos_clone.borrow_mut() = Some((x, y));
            }
        }) as Box<dyn FnMut(TouchEvent)>);

        canvas
            .add_event_listener_with_callback("touchstart", touchstart_closure.as_ref().unchecked_ref())
            .unwrap();
        touchstart_closure.forget();

        // TouchMove: Prevent default to avoid scrolling
        let touchmove_closure = Closure::wrap(Box::new(move |event: TouchEvent| {
            event.prevent_default();
        }) as Box<dyn FnMut(TouchEvent)>);

        canvas
            .add_event_listener_with_callback("touchmove", touchmove_closure.as_ref().unchecked_ref())
            .unwrap();
        touchmove_closure.forget();

        // TouchEnd: Detect swipe direction
        let touch_start_pos_clone = touch_start_pos.clone();
        let pending_input_clone = pending_input.clone();
        let touchend_closure = Closure::wrap(Box::new(move |event: TouchEvent| {
            event.prevent_default();

            // Get the touch that just ended
            if let Some(touch) = event.changed_touches().item(0) {
                let end_x = touch.client_x() as f64;
                let end_y = touch.client_y() as f64;

                // Check if we have a start position
                if let Some((start_x, start_y)) = *touch_start_pos_clone.borrow() {
                    let dx = end_x - start_x;
                    let dy = end_y - start_y;

                    // Calculate absolute distances
                    let abs_dx = dx.abs();
                    let abs_dy = dy.abs();

                    // Determine if swipe was strong enough and which direction
                    let input = if abs_dx > SWIPE_THRESHOLD || abs_dy > SWIPE_THRESHOLD {
                        // Primary direction is the one with larger delta
                        if abs_dx > abs_dy {
                            // Horizontal swipe
                            if dx > 0.0 {
                                Some(Input::Direction(Direction::Right))
                            } else {
                                Some(Input::Direction(Direction::Left))
                            }
                        } else {
                            // Vertical swipe
                            if dy > 0.0 {
                                Some(Input::Direction(Direction::Down))
                            } else {
                                Some(Input::Direction(Direction::Up))
                            }
                        }
                    } else {
                        None
                    };

                    // If we detected a valid swipe, register input and vibrate
                    if let Some(input) = input {
                        *pending_input_clone.borrow_mut() = Some(input);

                        // Haptic feedback (vibrate for 50ms)
                        // Try to vibrate - this will fail silently if not supported
                        if let Some(window) = web_sys::window() {
                            let navigator = window.navigator();
                            let _ = js_sys::Reflect::get(&navigator, &JsValue::from_str("vibrate"))
                                .ok()
                                .and_then(|vibrate_fn| {
                                    if vibrate_fn.is_function() {
                                        let vibrate = vibrate_fn.dyn_ref::<js_sys::Function>()?;
                                        let _ = vibrate.call1(&navigator, &JsValue::from_f64(50.0));
                                    }
                                    Some(())
                                });
                        }
                    }

                    // Clear touch start position
                    *touch_start_pos_clone.borrow_mut() = None;
                }
            }
        }) as Box<dyn FnMut(TouchEvent)>);

        canvas
            .add_event_listener_with_callback("touchend", touchend_closure.as_ref().unchecked_ref())
            .unwrap();
        touchend_closure.forget();

        // TouchCancel: Clear state if touch is cancelled
        let touchcancel_closure = Closure::wrap(Box::new(move |event: TouchEvent| {
            event.prevent_default();
            *touch_start_pos.borrow_mut() = None;
        }) as Box<dyn FnMut(TouchEvent)>);

        canvas
            .add_event_listener_with_callback("touchcancel", touchcancel_closure.as_ref().unchecked_ref())
            .unwrap();
        touchcancel_closure.forget();
    }

    fn current_time(&self) -> f64 {
        self.window.performance().unwrap().now()
    }

    fn calculate_interpolation_alpha(&self) -> f64 {
        let now = self.current_time();
        let elapsed = now - self.last_update_time;
        (elapsed / GAME_UPDATE_INTERVAL).min(1.0)
    }

    fn lerp(a: i32, b: i32, alpha: f64) -> f64 {
        a as f64 + (b - a) as f64 * alpha
    }

    fn detect_board_changes(&self, game: &Game) -> Vec<(i32, i32)> {
        let mut newly_filled = Vec::new();

        if let Some(prev) = &self.last_game_snapshot {
            // If board hash changed, scan for new filled cells
            let current_hash = GameSnapshot::hash_board(&game.board);
            if current_hash != prev.board_hash {
                for y in 0..game.height {
                    for x in 0..game.width {
                        if game.cell_at(x, y) == Cell::Filled {
                            // This is a newly filled cell (from trail completion)
                            newly_filled.push((x, y));
                        }
                    }
                }
            }
        }

        newly_filled
    }

    fn draw_cell(&self, x: i32, y: i32, color: &str) {
        self.context.set_fill_style_str(color);
        self.context.fill_rect(
            x as f64 * CELL_SIZE,
            y as f64 * CELL_SIZE,
            CELL_SIZE,
            CELL_SIZE,
        );
    }

    fn draw_cell_f64(&self, x: f64, y: f64, color: &str) {
        self.context.set_fill_style_str(color);
        self.context.fill_rect(
            x * CELL_SIZE,
            y * CELL_SIZE,
            CELL_SIZE,
            CELL_SIZE,
        );
    }

    fn draw_board(&self, game: &Game) {
        // Draw all cells
        for y in 0..game.height {
            for x in 0..game.width {
                let cell = game.cell_at(x, y);
                let color = match cell {
                    Cell::Empty => COLOR_EMPTY,
                    Cell::Filled => COLOR_FILLED,
                    Cell::Trail => COLOR_TRAIL,
                };
                self.draw_cell(x, y, color);
            }
        }
    }

    fn draw_fill_animation(&self, _game: &Game) {
        if let Some(anim) = &self.fill_animation {
            let now = self.current_time();
            let progress = anim.progress(now);

            // Easing function (ease-out-cubic)
            let eased = 1.0 - (1.0 - progress).powi(3);

            // Animate color from trail yellow to filled blue
            let r = (0x00 as f64 + (0xFF - 0x00) as f64 * (1.0 - eased)) as u8;
            let g = (0x00 as f64 + (0xFF - 0x00) as f64 * (1.0 - eased)) as u8;
            let b = (0xAA as f64 + (0x55 - 0xAA) as f64 * (1.0 - eased)) as u8;
            let color = format!("#{:02X}{:02X}{:02X}", r, g, b);

            // Draw animated cells based on flood progress
            let cells_to_show = (anim.cells_to_animate.len() as f64 * eased) as usize;
            for i in 0..cells_to_show {
                if i < anim.cells_to_animate.len() {
                    let (x, y) = anim.cells_to_animate[i];
                    self.draw_cell(x, y, &color);
                }
            }
        }
    }

    fn draw_ball_trails(&self) {
        for trail in &self.ball_trails {
            let positions = trail.positions();
            for (i, (x, y)) in positions.iter().enumerate() {
                let alpha = 1.0 - (i as f64 / positions.len() as f64);
                let alpha_hex = (alpha * 255.0) as u8;
                let color = format!("#{:02X}5555{:02X}", 0xFF, alpha_hex);

                self.context.set_fill_style_str(&color);
                self.context.fill_rect(
                    x * CELL_SIZE + CELL_SIZE * 0.25,
                    y * CELL_SIZE + CELL_SIZE * 0.25,
                    CELL_SIZE * 0.5,
                    CELL_SIZE * 0.5,
                );
            }
        }
    }

    fn draw_player(&self, x: f64, y: f64) {
        self.draw_cell_f64(x, y, COLOR_PLAYER);

        // Draw "@" symbol
        self.context.set_fill_style_str("#000000");
        self.context.set_font("12px monospace");
        self.context.set_text_align("center");
        self.context.set_text_baseline("middle");
        self.context
            .fill_text(
                "@",
                x * CELL_SIZE + CELL_SIZE / 2.0,
                y * CELL_SIZE + CELL_SIZE / 2.0 + 1.0,
            )
            .unwrap();
    }

    fn draw_ball(&self, x: f64, y: f64) {
        self.draw_cell_f64(x, y, COLOR_BALL);

        // Draw "()" symbol
        self.context.set_fill_style_str("#000000");
        self.context.set_font("10px monospace");
        self.context.set_text_align("center");
        self.context.set_text_baseline("middle");
        self.context
            .fill_text(
                "()",
                x * CELL_SIZE + CELL_SIZE / 2.0,
                y * CELL_SIZE + CELL_SIZE / 2.0 + 1.0,
            )
            .unwrap();
    }

    fn draw_ui(&self, game: &Game) {
        let y_offset = (game.height as f64 * CELL_SIZE) + 10.0;

        self.context.set_fill_style_str(COLOR_UI);
        self.context.set_font("14px monospace");
        self.context.set_text_align("left");
        self.context.set_text_baseline("top");

        let info = format!(
            "Level: {}  Score: {}  Filled: {:.1}%  Target: {:.0}%",
            game.level,
            game.score,
            game.filled_percentage * 100.0,
            game.target_percentage * 100.0
        );
        self.context.fill_text(&info, 5.0, y_offset).unwrap();

        let controls = "Controls: Arrow Keys / Swipe | Q: Quit | R: Restart";
        self.context.fill_text(controls, 5.0, y_offset + 20.0).unwrap();

        match game.state {
            GameState::Won => {
                self.context.set_fill_style_str("#55FF55");
                self.context
                    .fill_text("YOU WIN! Press SPACE for next level", 5.0, y_offset + 40.0)
                    .unwrap();
            }
            GameState::Lost => {
                self.context.set_fill_style_str("#FF5555");
                self.context
                    .fill_text("GAME OVER! Press R to restart", 5.0, y_offset + 40.0)
                    .unwrap();
            }
            GameState::Playing => {}
        }
    }
}

impl Renderer for WebRenderer {
    fn init(&mut self) -> io::Result<()> {
        // Setup input listeners
        self.setup_keyboard_listener();
        self.setup_touch_listeners();

        // Initialize time
        self.last_update_time = self.current_time();
        self.last_render_time = self.current_time();

        // Canvas size will be set properly on first render based on game dimensions

        Ok(())
    }

    fn render(&mut self, game: &Game) -> io::Result<()> {
        let now = self.current_time();

        // Frame rate limiting
        if now - self.last_render_time < TARGET_FRAME_TIME {
            return Ok(());
        }
        self.last_render_time = now;

        // Check if game updated (board hash changed)
        let game_updated = if let Some(prev) = &self.last_game_snapshot {
            GameSnapshot::hash_board(&game.board) != prev.board_hash ||
            game.player.position != prev.player_pos
        } else {
            true
        };

        // If game updated, capture snapshot and trigger effects
        if game_updated {
            // Detect newly filled cells for animation
            let newly_filled = self.detect_board_changes(game);
            if !newly_filled.is_empty() && self.fill_animation.is_none() {
                self.fill_animation = Some(FloodFillAnimation::new(newly_filled, now));
            }

            // Update snapshot
            self.last_game_snapshot = Some(GameSnapshot::from_game(game));
            self.last_update_time = now;

            // Initialize ball trails if needed
            while self.ball_trails.len() < game.balls.len() {
                self.ball_trails.push(BallTrail::new());
            }
        }

        // Set canvas size based on game dimensions
        // Display size (CSS pixels)
        let display_width = (game.width as f64 * CELL_SIZE) as u32;
        let display_height = (game.height as f64 * CELL_SIZE + 80.0) as u32;

        // Internal resolution (actual pixels, scaled for high DPI)
        let pixel_width = (display_width as f64 * self.device_pixel_ratio) as u32;
        let pixel_height = (display_height as f64 * self.device_pixel_ratio) as u32;

        if self.canvas.width() != pixel_width || self.canvas.height() != pixel_height {
            // Set internal resolution
            self.canvas.set_width(pixel_width);
            self.canvas.set_height(pixel_height);

            // Set CSS display size
            let element: &HtmlElement = self.canvas.unchecked_ref();
            element.style().set_property("width", &format!("{}px", display_width)).unwrap();
            element.style().set_property("height", &format!("{}px", display_height)).unwrap();

            // Reset transform and scale context to match device pixel ratio
            // This is needed because setting canvas width/height resets the context
            self.context.set_image_smoothing_enabled(false);
            self.context.scale(self.device_pixel_ratio, self.device_pixel_ratio).unwrap();
        }

        // Clear canvas (using display coordinates)
        self.context.clear_rect(0.0, 0.0, display_width as f64, display_height as f64);

        // Draw board (static)
        self.draw_board(game);

        // Draw fill animation if active
        if let Some(anim) = &self.fill_animation {
            if anim.is_complete(now) {
                self.fill_animation = None; // Animation complete
            } else {
                self.draw_fill_animation(game);
            }
        }

        // Calculate interpolation alpha
        let alpha = self.calculate_interpolation_alpha();

        // Draw balls with interpolation and trails
        if let Some(prev) = &self.last_game_snapshot {
            for (i, ball) in game.balls.iter().enumerate() {
                if i < prev.ball_positions.len() {
                    let prev_ball = &prev.ball_positions[i];
                    let bx = Self::lerp(prev_ball.x, ball.position.x, alpha);
                    let by = Self::lerp(prev_ball.y, ball.position.y, alpha);

                    // Update trail with current ball grid position for discontinuity detection
                    if i < self.ball_trails.len() {
                        self.ball_trails[i].add_position(bx, by, (ball.position.x, ball.position.y));
                    }
                }
            }
        }

        // Draw ball trails
        self.draw_ball_trails();

        // Draw balls at interpolated positions
        if let Some(prev) = &self.last_game_snapshot {
            for (i, ball) in game.balls.iter().enumerate() {
                if i < prev.ball_positions.len() {
                    let prev_ball = &prev.ball_positions[i];
                    let bx = Self::lerp(prev_ball.x, ball.position.x, alpha);
                    let by = Self::lerp(prev_ball.y, ball.position.y, alpha);
                    self.draw_ball(bx, by);
                }
            }
        }

        // Draw player at interpolated position
        if let Some(prev) = &self.last_game_snapshot {
            let px = Self::lerp(prev.player_pos.x, game.player.position.x, alpha);
            let py = Self::lerp(prev.player_pos.y, game.player.position.y, alpha);
            self.draw_player(px, py);
        } else {
            self.draw_player(game.player.position.x as f64, game.player.position.y as f64);
        }

        // Draw UI
        self.draw_ui(game);

        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        // No cleanup needed for web
        Ok(())
    }

    fn poll_input(&mut self) -> io::Result<Option<Input>> {
        Ok(self.pending_input.borrow_mut().take())
    }
}

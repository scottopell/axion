use std::collections::VecDeque;

/// Ball trail for motion blur effect
/// Separated from renderer for testing
#[derive(Clone, Debug)]
pub struct BallTrail {
    positions: VecDeque<(f64, f64)>,
    max_length: usize,
    last_ball_position: Option<(i32, i32)>, // Grid position of ball at last game update
}

impl BallTrail {
    pub fn new() -> Self {
        Self {
            positions: VecDeque::new(),
            max_length: 6,
            last_ball_position: None,
        }
    }

    /// Add a position to the trail
    /// Returns true if position was added, false if trail was cleared due to discontinuity
    pub fn add_position(&mut self, x: f64, y: f64, current_ball_grid_pos: (i32, i32)) -> bool {
        // Check if ball moved significantly (more than 1 grid cell)
        // In the actual game, balls move 1 cell per update, so any jump > 1 is a discontinuity
        // This indicates a bounce or game state change, so we should clear the trail
        if let Some(last_pos) = self.last_ball_position {
            let dx = (current_ball_grid_pos.0 - last_pos.0).abs();
            let dy = (current_ball_grid_pos.1 - last_pos.1).abs();

            // If ball jumped more than 1 cell, clear trail (ball bounced or state changed)
            if dx > 1 || dy > 1 {
                self.positions.clear();
                self.last_ball_position = Some(current_ball_grid_pos);
                self.positions.push_front((x, y));
                return false;
            }
        }

        self.last_ball_position = Some(current_ball_grid_pos);
        self.positions.push_front((x, y));

        while self.positions.len() > self.max_length {
            self.positions.pop_back();
        }

        true
    }

    /// Clear all trail positions
    pub fn clear(&mut self) {
        self.positions.clear();
        self.last_ball_position = None;
    }

    /// Get all positions for rendering
    pub fn positions(&self) -> &VecDeque<(f64, f64)> {
        &self.positions
    }

    /// Check if all trail positions are within reasonable distance of current ball
    /// Returns (is_valid, max_distance_found)
    pub fn validate_trail_distance(&self, current_ball_pos: (f64, f64), max_allowed_distance: f64) -> (bool, f64) {
        let mut max_distance: f64 = 0.0;

        for (x, y) in &self.positions {
            let dx = x - current_ball_pos.0;
            let dy = y - current_ball_pos.1;
            let distance = (dx * dx + dy * dy).sqrt();
            max_distance = max_distance.max(distance);
        }

        (max_distance <= max_allowed_distance, max_distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trail_basic() {
        let mut trail = BallTrail::new();

        // Add positions in a straight line
        trail.add_position(1.0, 1.0, (1, 1));
        trail.add_position(2.0, 1.0, (2, 1));
        trail.add_position(3.0, 1.0, (3, 1));

        assert_eq!(trail.positions().len(), 3);

        // Verify distance constraint
        let (valid, _) = trail.validate_trail_distance((3.0, 1.0), 5.0);
        assert!(valid, "Trail should be within 5 units of ball");
    }

    #[test]
    fn test_trail_clears_on_discontinuity() {
        let mut trail = BallTrail::new();

        // Add positions moving right
        trail.add_position(1.0, 1.0, (1, 1));
        trail.add_position(2.0, 1.0, (2, 1));
        trail.add_position(3.0, 1.0, (3, 1));

        assert_eq!(trail.positions().len(), 3);

        // Ball suddenly at (10, 1) - should clear trail
        let cleared = trail.add_position(10.0, 1.0, (10, 1));

        assert!(!cleared, "Trail should be cleared due to discontinuity");
        assert_eq!(trail.positions().len(), 1, "Trail should only have new position");
    }

    #[test]
    fn test_trail_max_length() {
        let mut trail = BallTrail::new();

        // Add more positions than max_length
        for i in 0..10 {
            trail.add_position(i as f64, 1.0, (i, 1));
        }

        assert_eq!(trail.positions().len(), 6, "Trail should respect max_length");
    }

    #[test]
    fn test_trail_distance_invariant_after_bounce() {
        let mut trail = BallTrail::new();

        // Simulate ball moving right (continuous movement, 1 cell per step)
        for i in 0..5 {
            trail.add_position(i as f64, 5.0, (i, 5));
        }

        // Ball bounces - grid position jumps by 2 cells (from 4 to 6)
        // This should clear the trail since it's > 1 cell jump
        let cleared = trail.add_position(6.0, 5.0, (6, 5));

        assert!(!cleared, "Trail should be cleared due to >1 cell jump");

        // After bounce/clear, trail should only have the new position
        let (valid, distance) = trail.validate_trail_distance((6.0, 5.0), 1.0);
        assert!(valid, "Trail distance {} should be ~0 after clear", distance);
        assert_eq!(trail.positions().len(), 1, "Trail should only have 1 position after clear");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_trail_always_near_ball(
            moves in prop::collection::vec((0i32..40, 0i32..20), 10..100)
        ) {
            let mut trail = BallTrail::new();

            for (target_x, target_y) in moves {
                // Add interpolated position
                let fx = target_x as f64;
                let fy = target_y as f64;
                trail.add_position(fx, fy, (target_x, target_y));

                // Verify invariant: all trail positions should be within reasonable distance
                // Max distance should be roughly max_length cells (since ball moves 1 cell per update)
                let (valid, distance) = trail.validate_trail_distance((fx, fy), 10.0);
                prop_assert!(valid, "Trail distance {} exceeded 10 units from ball at ({}, {})", distance, target_x, target_y);
            }
        }

        #[test]
        fn prop_trail_clears_on_large_jump(
            jump_size in 2i32..10,
            initial_trail_length in 2usize..=6
        ) {
            let mut trail = BallTrail::new();

            // Build up a trail (at least 2 positions so we can see a decrease)
            for i in 0..initial_trail_length {
                trail.add_position(i as f64, 5.0, (i as i32, 5));
            }

            let trail_len_before = trail.positions().len();
            prop_assert!(trail_len_before >= 2, "Need at least 2 positions to test");

            // Make a large jump (> 1 cell)
            let jump_target = initial_trail_length as i32 + jump_size;
            let cleared = trail.add_position(jump_target as f64, 5.0, (jump_target, 5));

            // Trail should be cleared (returns false) and have only 1 position now
            prop_assert!(!cleared, "add_position should return false when clearing");
            let trail_len_after = trail.positions().len();
            prop_assert_eq!(trail_len_after, 1,
                "Trail should have 1 position after large jump (had {} before)",
                trail_len_before);
        }
    }
}

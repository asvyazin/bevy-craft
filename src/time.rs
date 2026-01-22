use bevy::prelude::*;

/// Resource for managing the day/night cycle
#[derive(Resource, Debug, Clone)]
pub struct GameTime {
    /// Current time in seconds (0-86400 for 24-hour cycle)
    pub current_time: f32,
    /// Time multiplier (1.0 = real-time, higher = faster)
    pub time_scale: f32,
    /// Whether the time should progress
    pub time_paused: bool,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            current_time: 43200.0, // Start at noon (12:00 PM)
            time_scale: 60.0,      // 60x faster than real-time for testing
            time_paused: false,
        }
    }
}

#[allow(dead_code)]
impl GameTime {
    /// Create a new GameTime instance
    pub fn new(start_time: f32, time_scale: f32) -> Self {
        Self {
            current_time: start_time,
            time_scale,
            time_paused: false,
        }
    }

    /// Get current time in hours (0-24)
    pub fn current_hour(&self) -> f32 {
        (self.current_time / 3600.0) % 24.0
    }

    /// Get current time in minutes (0-60)
    pub fn current_minute(&self) -> f32 {
        (self.current_time / 60.0) % 60.0
    }

    /// Get current time as normalized value (0.0-1.0) for day/night cycle
    pub fn time_of_day_normalized(&self) -> f32 {
        self.current_hour() / 24.0
    }

    /// Check if it's currently day time (6:00 AM to 6:00 PM)
    pub fn is_day(&self) -> bool {
        let hour = self.current_hour();
        hour >= 6.0 && hour < 18.0
    }

    /// Check if it's currently night time
    pub fn is_night(&self) -> bool {
        !self.is_day()
    }

    /// Get sun position in radians for rendering (0 = midnight, π = noon)
    pub fn sun_angle_radians(&self) -> f32 {
        let normalized_time = self.time_of_day_normalized();
        normalized_time * std::f32::consts::PI * 2.0 - std::f32::consts::PI / 2.0
    }

    /// Get moon position in radians for rendering (opposite of sun)
    pub fn moon_angle_radians(&self) -> f32 {
        self.sun_angle_radians() + std::f32::consts::PI
    }

    /// Update time based on delta time
    pub fn update(&mut self, delta_secs: f32) {
        if !self.time_paused {
            self.current_time += delta_secs * self.time_scale;
            // Wrap around after 24 hours
            if self.current_time >= 86400.0 {
                self.current_time -= 86400.0;
            }
        }
    }

    /// Format time as HH:MM string
    pub fn format_time(&self) -> String {
        let hours = (self.current_time / 3600.0) as u32 % 24;
        let minutes = (self.current_time / 60.0) as u32 % 60;
        format!("{:02}:{:02}", hours, minutes)
    }
}

/// System to update game time
pub fn update_game_time(time: Res<Time>, mut game_time: ResMut<GameTime>) {
    game_time.update(time.delta_secs());
}

/// System to display current game time (for debugging)
pub fn display_game_time(game_time: Res<GameTime>) {
    if game_time.current_time % 10.0 < 0.1 {
        // Display every 10 seconds
        println!(
            "🕒 Game Time: {} (Hour: {:.1}, Day: {})",
            game_time.format_time(),
            game_time.current_hour(),
            if game_time.is_day() { "Day" } else { "Night" }
        );
    }
}

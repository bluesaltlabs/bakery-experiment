use bevy::prelude::*;

#[derive(Resource)]
pub struct ShiftState {
    pub time_remaining: f32,
    pub cases_completed: u32,
    pub target_cases: u32,
    pub game_over: bool,
    pub victory: bool,
}

impl ShiftState {
    pub fn new() -> Self {
        ShiftState {
            time_remaining: 120.0,
            cases_completed: 0,
            target_cases: 3,
            game_over: false,
            victory: false,
        }
    }
}

#[derive(Resource)]
pub struct MovementCooldown(pub Timer);

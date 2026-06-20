use std::collections::HashMap;
use bevy::prelude::*;
use crate::components::{Direction, GridPos, NpcKind};
use crate::level::{MAP_WIDTH, MAP_HEIGHT};

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
            time_remaining: 300.0,
            cases_completed: 0,
            target_cases: 10,
            game_over: false,
            victory: false,
        }
    }
}

#[derive(Resource)]
pub struct MovementCooldown(pub Timer);

#[derive(Resource)]
pub struct GridVisible(pub bool);

#[derive(Resource)]
pub struct ConveyorTimerResource(pub Timer);

#[derive(Resource)]
pub struct LevelData {
    pub tiles: [[u8; MAP_WIDTH]; MAP_HEIGHT],
    pub conveyor_dirs: HashMap<(usize, usize), Direction>,
    pub npcs: Vec<NpcSpawnData>,
}

impl LevelData {
    pub fn new() -> Self {
        use crate::level::LEVEL_DATA;
        let mut conveyor_dirs = HashMap::new();
        for (row, line) in LEVEL_DATA.iter().enumerate() {
            for (col, &tile) in line.iter().enumerate() {
                if tile == crate::level::CONVEYOR {
                    conveyor_dirs.insert((col, row), Direction::Down);
                }
            }
        }
        LevelData {
            tiles: LEVEL_DATA,
            conveyor_dirs,
            npcs: vec![
                NpcSpawnData { kind: NpcKind::ConveyorLoader, pos: GridPos { x: 4, y: 4 }, facing: Direction::Left },
                NpcSpawnData { kind: NpcKind::OvenHauler, pos: GridPos { x: 5, y: 2 }, facing: Direction::Left },
                NpcSpawnData { kind: NpcKind::PackerHauler, pos: GridPos { x: 7, y: 5 }, facing: Direction::Right },
            ],
        }
    }
}

#[derive(Clone)]
pub struct NpcSpawnData {
    pub kind: NpcKind,
    pub pos: GridPos,
    pub facing: Direction,
}

#[derive(Resource)]
pub struct EditorMode(pub bool);

#[derive(Resource)]
pub struct SelectedTile(pub u8);

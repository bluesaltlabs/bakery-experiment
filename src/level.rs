use bevy::prelude::*;
use crate::components::*;

pub const TILE_SIZE: f32 = 64.0;
pub const MAP_WIDTH: usize = 16;
pub const MAP_HEIGHT: usize = 10;
pub const PLAYER_START: (usize, usize) = (7, 5);

const WALL: u8 = 1;
const SOURCE: u8 = 2;
const FORMER: u8 = 3;
const OVEN: u8 = 4;
const PACKER: u8 = 5;
const PALLETIZER: u8 = 6;

const LEVEL_DATA: [[u8; MAP_WIDTH]; MAP_HEIGHT] = [
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,6,0,0,1],
    [1,0,0,2,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,3,0,0,0,0,0,0,4,0,5,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
];

pub fn grid_to_world(pos: GridPos) -> Vec3 {
    Vec3::new(
        pos.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        0.0,
    )
}

pub fn setup_level(commands: &mut Commands) {
    for (row, line) in LEVEL_DATA.iter().enumerate() {
        for (col, &tile) in line.iter().enumerate() {
            let y = (MAP_HEIGHT - 1 - row) as i32;
            let x = col as i32;
            let pos = GridPos { x, y };

            match tile {
                WALL => {
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::srgb(0.3, 0.3, 0.35),
                                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                                ..default()
                            },
                            transform: Transform::from_translation(grid_to_world(pos)),
                            ..default()
                        },
                        pos,
                        Solid,
                        GameEntity,
                    ));
                }
                SOURCE => {
                    spawn_station(commands, pos, StationKind::Source);
                }
                FORMER => {
                    spawn_station(commands, pos, StationKind::Former);
                }
                OVEN => {
                    spawn_station(commands, pos, StationKind::Oven);
                }
                PACKER => {
                    spawn_station(commands, pos, StationKind::Packer);
                }
                PALLETIZER => {
                    spawn_station(commands, pos, StationKind::Palletizer);
                }
                _ => {
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::srgb(0.5, 0.5, 0.55),
                                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                                ..default()
                            },
                            transform: Transform::from_translation(grid_to_world(pos)),
                            ..default()
                        },
                        pos,
                        GameEntity,
                    ));
                }
            }
        }
    }
}

fn spawn_station(commands: &mut Commands, pos: GridPos, kind: StationKind) {
    let (accepted_kind, output_kind, process_duration, spawn_interval) = match kind {
        StationKind::Source => (ItemKind::DoughBatch, ItemKind::DoughBatch, 0.0, 6.0),
        StationKind::Former => (ItemKind::DoughBatch, ItemKind::RawCrustTray, 3.0, 0.0),
        StationKind::Oven => (ItemKind::RawCrustTray, ItemKind::BakedCrustTray, 5.0, 0.0),
        StationKind::Packer => (ItemKind::BakedCrustTray, ItemKind::Case, 4.0, 0.0),
        StationKind::Palletizer => (ItemKind::Case, ItemKind::Case, 0.0, 0.0),
    };

    let world_pos = grid_to_world(pos);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: kind.color(),
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(world_pos),
            ..default()
        },
        pos,
        Station {
            kind,
            accepted_kind,
            output_kind,
            process_duration,
            timer: 0.0,
            busy: false,
            has_output: false,
            packer_count: 0,
            spawn_timer: 0.0,
            spawn_interval,
        },
        GameEntity,
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                kind.label(),
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_translation(Vec3::new(
                world_pos.x,
                world_pos.y - TILE_SIZE / 2.0 - 10.0,
                1.0,
            )),
            ..default()
        },
        GameEntity,
    ));
}

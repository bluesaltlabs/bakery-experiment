use bevy::prelude::*;
use crate::components::*;
use crate::resources::GridVisible;

pub const TILE_SIZE: f32 = 64.0;
pub const MAP_WIDTH: usize = 12;
pub const MAP_HEIGHT: usize = 10;
pub const PLAYER_START: (usize, usize) = (6, 4); // col 6, row 4 in LEVEL_DATA => grid (6, 5), floor tile right of wall

const WALL: u8 = 1;
const SOURCE: u8 = 2;
const FORMER: u8 = 3;
const OVEN: u8 = 4;
const PACKER: u8 = 5;
const PALLETIZER: u8 = 6;

// Layout (row 0 = top, y=9):
//   WWWWWWWWWWWW
//   W..S.......W
//   W..........W
//   W..........W
//   W....W..P..W   wall at (5,5), Packer at (8,5)
//   W....W.....W   wall at (5,4)
//   W..F.......W
//   W...O......W
//   W....PL....W
//   WWWWWWWWWWWW
const LEVEL_DATA: [[u8; MAP_WIDTH]; MAP_HEIGHT] = [
    [1,1,1,1,1,1,1,1,1,1,1,1], // y=9 (top)
    [1,0,0,2,0,0,0,0,0,0,0,1], // y=8  Source at (3,8)
    [1,0,0,0,0,0,0,0,0,0,0,1], // y=7
    [1,0,0,0,0,0,0,0,0,0,0,1], // y=6
    [1,0,0,0,0,1,0,0,5,0,0,1], // y=5  wall at (5,5), Packer at (8,5)
    [1,0,0,0,0,1,0,0,0,0,0,1], // y=4  wall at (5,4)
    [1,0,0,3,0,0,0,0,0,0,0,1], // y=3  Former at (3,3)
    [1,0,0,0,4,0,0,0,0,0,0,1], // y=2  Oven at (4,2)
    [1,0,0,0,0,0,6,0,0,0,0,1], // y=1  Palletizer at (6,1)
    [1,1,1,1,1,1,1,1,1,1,1,1], // y=0 (bottom)
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

    spawn_gridlines(commands);
}

fn spawn_station(commands: &mut Commands, pos: GridPos, kind: StationKind) {
    let (accepted_kind, output_kind, process_duration, spawn_interval) = match kind {
        StationKind::Source => (ItemKind::DoughBatch, ItemKind::DoughBatch, 0.0, 4.0),
        StationKind::Former => (ItemKind::DoughBatch, ItemKind::RawCrustTray, 3.0, 0.0),
        StationKind::Oven => (ItemKind::RawCrustTray, ItemKind::BakedCrustTray, 5.0, 0.0),
        StationKind::Packer => (ItemKind::BakedCrustTray, ItemKind::Case, 2.0, 0.0),
        StationKind::Palletizer => (ItemKind::Case, ItemKind::Case, 0.0, 0.0),
    };

    let inputs_needed = match kind {
        StationKind::Packer => 3,
        _ => 1,
    };

    let world_pos = grid_to_world(pos);

    let station_entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: kind.color_idle(),
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
            inputs_needed,
            spawn_timer: 0.0,
            spawn_interval,
        },
        GameEntity,
    )).id();

    let label_prefix = kind.label();
    let label_suffix = if kind == StationKind::Packer {
        format!(" 0/{}", inputs_needed)
    } else {
        String::new()
    };

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("{}{}", label_prefix, label_suffix),
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
        StationLabel { station_entity },
        GameEntity,
    ));
}

const GRID_LINE_COLOR: Color = Color::srgb(0.35, 0.35, 0.4);
const GRID_LINE_WIDTH: f32 = 2.0;

fn spawn_gridlines(commands: &mut Commands) {
    let map_w = MAP_WIDTH as f32 * TILE_SIZE;
    let map_h = MAP_HEIGHT as f32 * TILE_SIZE;

    for col in 0..=MAP_WIDTH {
        let x = col as f32 * TILE_SIZE;
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: GRID_LINE_COLOR,
                    custom_size: Some(Vec2::new(GRID_LINE_WIDTH, map_h)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(x, map_h / 2.0, 0.5)),
                ..default()
            },
            GameEntity,
            GridLine,
        ));
    }

    for row in 0..=MAP_HEIGHT {
        let y = row as f32 * TILE_SIZE;
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: GRID_LINE_COLOR,
                    custom_size: Some(Vec2::new(map_w, GRID_LINE_WIDTH)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(map_w / 2.0, y, 0.5)),
                ..default()
            },
            GameEntity,
            GridLine,
        ));
    }
}

pub fn toggle_grid(
    keys: Res<ButtonInput<KeyCode>>,
    mut grid_visible: ResMut<GridVisible>,
    mut query: Query<&mut Visibility, With<GridLine>>,
) {
    if !keys.just_pressed(KeyCode::KeyG) {
        return;
    }

    grid_visible.0 = !grid_visible.0;
    let vis = if grid_visible.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut v in query.iter_mut() {
        *v = vis;
    }
}

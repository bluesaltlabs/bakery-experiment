use bevy::prelude::*;
use crate::components::*;
use crate::mobile::MobileInput;
use crate::resources::{GridVisible, LevelData};

pub const TILE_SIZE: f32 = 64.0;
pub const MAP_WIDTH: usize = 30;
pub const MAP_HEIGHT: usize = 20;
pub const PLAYER_START: (usize, usize) = (1, 18);

pub const WALL: u8 = 1;
pub const SOURCE: u8 = 2;
pub const FORMER: u8 = 3;
pub const OVEN: u8 = 4;
pub const PACKER: u8 = 5;
pub const PALLETIZER: u8 = 6;
pub const TABLE: u8 = 7;
pub const CONVEYOR: u8 = 8;

pub const LEVEL_DATA: [[u8; MAP_WIDTH]; MAP_HEIGHT] = [
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,8,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,8,0,0,7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,8,0,0,0,0,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,8,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,6,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
];

pub fn grid_to_world(pos: GridPos) -> Vec3 {
    Vec3::new(
        pos.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        0.0,
    )
}

pub fn setup_level(commands: &mut Commands, level_data: &LevelData) {
    for (row, line) in level_data.tiles.iter().enumerate() {
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
                TABLE => {
                    spawn_station(commands, pos, StationKind::Table);
                }
                CONVEYOR => {
                    let dir = level_data.conveyor_dirs
                        .get(&(col, row))
                        .copied()
                        .unwrap_or(crate::components::Direction::Down);
                    spawn_conveyor(commands, pos, dir);
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
        StationKind::Table => (ItemKind::DoughBatch, ItemKind::DoughBatch, 0.0, 0.0),
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

    if kind == StationKind::Table {
        commands.entity(station_entity).insert(TableMarker);
    }

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

fn spawn_conveyor(commands: &mut Commands, pos: GridPos, direction: crate::components::Direction) {
    use crate::components::Direction as LocalDir;
    let world_pos = grid_to_world(pos);
    let half = TILE_SIZE * 0.4;
    let bar_half = TILE_SIZE * 0.08;
    let (offset_x, offset_y, width, height) = match direction {
        LocalDir::Up => (0.0, half - bar_half, TILE_SIZE * 0.6, TILE_SIZE * 0.16),
        LocalDir::Down => (0.0, -(half - bar_half), TILE_SIZE * 0.6, TILE_SIZE * 0.16),
        LocalDir::Left => (-(half - bar_half), 0.0, TILE_SIZE * 0.16, TILE_SIZE * 0.6),
        LocalDir::Right => (half - bar_half, 0.0, TILE_SIZE * 0.16, TILE_SIZE * 0.6),
    };

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.25, 0.35, 0.45),
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(world_pos),
            ..default()
        },
        pos,
        ConveyorBelt { direction },
        GameEntity,
    ));

    let arrow_pos = Vec3::new(world_pos.x + offset_x, world_pos.y + offset_y, 0.05);
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.7, 0.9),
                custom_size: Some(Vec2::new(width, height)),
                ..default()
            },
            transform: Transform::from_translation(arrow_pos),
            ..default()
        },
        GameEntity,
        ConveyorArrow {
            direction,
            base: arrow_pos,
        },
    ));
}

const GRID_LINE_COLOR: Color = Color::srgb(0.35, 0.35, 0.4);
const GRID_LINE_WIDTH: f32 = 2.0;

fn spawn_gridlines(commands: &mut Commands) {
    let map_w = MAP_WIDTH as f32 * TILE_SIZE;
    let map_h = MAP_HEIGHT as f32 * TILE_SIZE;

    for i in 0..=MAP_WIDTH.max(MAP_HEIGHT) {
        if i <= MAP_WIDTH {
            let x = i as f32 * TILE_SIZE;
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
        if i <= MAP_HEIGHT {
            let y = i as f32 * TILE_SIZE;
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
}

pub fn spawn_item_entity(commands: &mut Commands, kind: ItemKind, position: Vec3) -> Entity {
    commands
        .spawn((
            Item { kind },
            SpriteBundle {
                sprite: Sprite {
                    color: kind.color(),
                    custom_size: Some(Vec2::new(TILE_SIZE * 0.45, TILE_SIZE * 0.45)),
                    ..default()
                },
                transform: Transform::from_translation(position),
                ..default()
            },
            GameEntity,
        ))
        .id()
}

pub fn toggle_grid(
    keys: Res<ButtonInput<KeyCode>>,
    mut mobile_input: ResMut<MobileInput>,
    mut grid_visible: ResMut<GridVisible>,
    mut query: Query<&mut Visibility, With<GridLine>>,
) {
    let toggle = keys.just_pressed(KeyCode::KeyG) || mobile_input.toggle_grid;
    mobile_input.toggle_grid = false;
    if !toggle {
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

use bevy::prelude::*;
use crate::components::{
    Carrying, ConveyorBelt, Direction, Facing, FloorTimer, GameEntity, GridPos, Item,
    ItemKind, Npc, NpcDirectionIndicator, NpcState, Player, Solid, Station, StationKind,
    TableMarker,
};
use crate::level::{grid_to_world, TILE_SIZE, MAP_WIDTH, MAP_HEIGHT};
use crate::resources::ShiftState;

const NPC_MOVE_COOLDOWN: f32 = 0.5;
const NPC_ACTION_COOLDOWN: f32 = 0.25;
const NPC_Z: f32 = 0.02;

pub fn spawn_npc(commands: &mut Commands) {
    let pos = GridPos { x: 4, y: 4 };
    let world_pos = grid_to_world(pos);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.5, 0.0),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.7, TILE_SIZE * 0.7)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                world_pos.x,
                world_pos.y,
                NPC_Z,
            )),
            ..default()
        },
        pos,
        Facing(Direction::Left),
        Carrying::empty(),
        Npc {
            state: NpcState::WaitingAtConveyor,
            move_timer: 0.0,
            action_timer: 0.0,
        },
        GameEntity,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.7, 0.3),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.7, TILE_SIZE * 0.15)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                world_pos.x,
                world_pos.y + TILE_SIZE * 0.35 - TILE_SIZE * 0.075,
                0.05,
            )),
            ..default()
        },
        NpcDirectionIndicator,
        GameEntity,
    ));
}

pub fn npc_ai(
    time: Res<Time>,
    shift: Res<ShiftState>,
    mut npc_query: Query<(Entity, &mut GridPos, &mut Facing, &mut Carrying, &mut Npc, &mut Transform)>,
    mut station_query: Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    item_on_ground_query: Query<(Entity, &Item, &GridPos), (Without<Npc>, Without<Player>)>,
    station_pos_query: Query<&GridPos, (With<Station>, Without<Npc>)>,
    conveyor_pos_query: Query<&GridPos, (With<ConveyorBelt>, Without<Npc>)>,
    solid_query: Query<&GridPos, (With<Solid>, Without<Item>, Without<Npc>)>,
    player_query: Query<&GridPos, (With<Player>, Without<Npc>)>,
    mut commands: Commands,
) {
    if shift.game_over {
        return;
    }

    let dt = time.delta_seconds();

    for (_entity, mut pos, mut facing, mut carrying, mut npc, mut transform) in npc_query.iter_mut() {
        npc.move_timer -= dt;
        npc.action_timer -= dt;

        match npc.state {
            NpcState::WaitingAtConveyor => {
                if npc.action_timer > 0.0 {
                    continue;
                }

                let conveyor_end = GridPos { x: 3, y: 4 };
                let has_product = item_on_ground_query
                    .iter()
                    .any(|(_, item, ip)| *ip == conveyor_end && item.kind == ItemKind::DoughBatch);

                if has_product {
                    facing.0 = Direction::Left;
                    npc.state = NpcState::PickingUp;
                } else {
                    npc.action_timer = NPC_ACTION_COOLDOWN;
                }
            }

            NpcState::PickingUp => {
                if npc.action_timer > 0.0 {
                    continue;
                }
                facing.0 = Direction::Left;
                let front_pos = GridPos { x: pos.x - 1, y: pos.y };

                let mut picked_up = false;
                if let Some((item_entity, item_kind)) = item_on_ground_query.iter().find_map(|(e, i, ip)| {
                    if *ip == front_pos && i.kind == ItemKind::DoughBatch {
                        Some((e, i.kind))
                    } else {
                        None
                    }
                }) {
                    if carrying.entity.is_none() {
                        commands.entity(item_entity).remove::<GridPos>();
                        commands.entity(item_entity).remove::<FloorTimer>();
                        carrying.entity = Some(item_entity);
                        carrying.kind = Some(item_kind);
                        picked_up = true;
                    }
                }

                if picked_up {
                    npc.state = NpcState::MovingToFormer;
                } else {
                    npc.action_timer = NPC_ACTION_COOLDOWN;
                    npc.state = NpcState::WaitingAtConveyor;
                }
            }

            NpcState::MovingToFormer => {
                let target = GridPos { x: 4, y: 3 };
                try_npc_move(
                    &mut pos, &mut transform, target,
                    &solid_query, &station_pos_query, &conveyor_pos_query, &player_query, &mut npc,
                );

                if pos.x == target.x && pos.y == target.y {
                    npc.state = NpcState::InsertingToFormer;
                }
            }

            NpcState::InsertingToFormer => {
                if npc.action_timer > 0.0 {
                    continue;
                }
                facing.0 = Direction::Left;
                let front_pos = GridPos { x: pos.x - 1, y: pos.y };

                let mut inserted = false;
                for (_station_entity, mut station, station_pos) in station_query.iter_mut() {
                    if *station_pos == front_pos && station.kind == StationKind::Former {
                        if carrying.kind == Some(ItemKind::DoughBatch)
                            && !station.busy
                            && !station.has_output
                        {
                            if let Some(carried_entity) = carrying.entity {
                                commands.entity(carried_entity).despawn();
                                carrying.entity = None;
                                carrying.kind = None;
                                station.busy = true;
                                station.timer = 0.0;
                                inserted = true;
                            }
                        }
                        break;
                    }
                }

                if inserted {
                    npc.state = NpcState::WaitingForFormer;
                } else {
                    npc.action_timer = NPC_ACTION_COOLDOWN;
                }
            }

            NpcState::WaitingForFormer => {
                if npc.action_timer > 0.0 {
                    continue;
                }
                let front_pos = GridPos { x: pos.x - 1, y: pos.y };

                for (_station_entity, station, station_pos) in station_query.iter_mut() {
                    if *station_pos == front_pos && station.kind == StationKind::Former {
                        if station.has_output && !station.busy {
                            npc.state = NpcState::CollectingFromFormer;
                        }
                        break;
                    }
                }

                npc.action_timer = NPC_ACTION_COOLDOWN;
            }

            NpcState::CollectingFromFormer => {
                if npc.action_timer > 0.0 {
                    continue;
                }
                facing.0 = Direction::Left;
                let front_pos = GridPos { x: pos.x - 1, y: pos.y };

                let mut collected = false;
                for (_station_entity, mut station, station_pos) in station_query.iter_mut() {
                    if *station_pos == front_pos && station.kind == StationKind::Former {
                        if station.has_output && carrying.entity.is_none() {
                            let item_entity = commands
                                .spawn((
                                    Item {
                                        kind: station.output_kind,
                                    },
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: station.output_kind.color(),
                                            custom_size: Some(Vec2::new(
                                                TILE_SIZE * 0.45,
                                                TILE_SIZE * 0.45,
                                            )),
                                            ..default()
                                        },
                                        transform: Transform::from_translation(Vec3::new(
                                            transform.translation.x,
                                            transform.translation.y,
                                            0.1,
                                        )),
                                        ..default()
                                    },
                                    GameEntity,
                                ))
                                .id();
                            carrying.entity = Some(item_entity);
                            carrying.kind = Some(station.output_kind);
                            station.has_output = false;
                            collected = true;
                            npc.state = NpcState::InsertingToOven;
                        }
                        break;
                    }
                }

                if !collected {
                    npc.action_timer = NPC_ACTION_COOLDOWN;
                }
            }

            NpcState::InsertingToOven => {
                if npc.action_timer > 0.0 {
                    continue;
                }
                facing.0 = Direction::Down;
                let front_pos = GridPos {
                    x: pos.x,
                    y: pos.y - 1,
                };

                let mut inserted = false;
                for (_station_entity, mut station, station_pos) in station_query.iter_mut() {
                    if *station_pos == front_pos && station.kind == StationKind::Oven {
                        if carrying.kind == Some(ItemKind::RawCrustTray)
                            && !station.busy
                            && !station.has_output
                        {
                            if let Some(carried_entity) = carrying.entity {
                                commands.entity(carried_entity).despawn();
                                carrying.entity = None;
                                carrying.kind = None;
                                station.busy = true;
                                station.timer = 0.0;
                            }
                            inserted = true;
                            npc.state = NpcState::ReturningToConveyor;
                        }
                        break;
                    }
                }

                if !inserted {
                    npc.action_timer = NPC_ACTION_COOLDOWN;
                }
            }

            NpcState::ReturningToConveyor => {
                let target = GridPos { x: 4, y: 4 };
                try_npc_move(
                    &mut pos, &mut transform, target,
                    &solid_query, &station_pos_query, &conveyor_pos_query, &player_query, &mut npc,
                );

                if pos.x == target.x && pos.y == target.y {
                    facing.0 = Direction::Left;
                    npc.state = NpcState::WaitingAtConveyor;
                }
            }
        }
    }
}

fn is_tile_blocked_for_npc(
    tile: GridPos,
    solid_query: &Query<&GridPos, (With<Solid>, Without<Item>, Without<Npc>)>,
    station_pos_query: &Query<&GridPos, (With<Station>, Without<Npc>)>,
    conveyor_pos_query: &Query<&GridPos, (With<ConveyorBelt>, Without<Npc>)>,
    player_query: &Query<&GridPos, (With<Player>, Without<Npc>)>,
) -> bool {
    if tile.x < 0 || tile.x >= MAP_WIDTH as i32 || tile.y < 0 || tile.y >= MAP_HEIGHT as i32 {
        return true;
    }

    solid_query.iter().any(|gp| *gp == tile)
        || station_pos_query.iter().any(|gp| *gp == tile)
        || conveyor_pos_query.iter().any(|gp| *gp == tile)
        || player_query.iter().any(|gp| *gp == tile)
}

fn try_npc_move(
    pos: &mut GridPos,
    transform: &mut Transform,
    target: GridPos,
    solid_query: &Query<&GridPos, (With<Solid>, Without<Item>, Without<Npc>)>,
    station_pos_query: &Query<&GridPos, (With<Station>, Without<Npc>)>,
    conveyor_pos_query: &Query<&GridPos, (With<ConveyorBelt>, Without<Npc>)>,
    player_query: &Query<&GridPos, (With<Player>, Without<Npc>)>,
    npc: &mut Npc,
) {
    if *pos == target {
        return;
    }

    if npc.move_timer > 0.0 {
        return;
    }

    let dx = target.x - pos.x;
    let dy = target.y - pos.y;

    let try_move = |dx: i32, dy: i32| -> Option<GridPos> {
        let new_pos = GridPos {
            x: pos.x + dx,
            y: pos.y + dy,
        };
        if !is_tile_blocked_for_npc(new_pos, solid_query, station_pos_query, conveyor_pos_query, player_query) {
            Some(new_pos)
        } else {
            None
        }
    };

    let moved = if dx.abs() >= dy.abs() {
        try_move(dx.signum(), 0).or_else(|| try_move(0, dy.signum()))
    } else {
        try_move(0, dy.signum()).or_else(|| try_move(dx.signum(), 0))
    };

    if let Some(new_pos) = moved {
        pos.x = new_pos.x;
        pos.y = new_pos.y;
        transform.translation = grid_to_world(new_pos);
        transform.translation.z = NPC_Z;
        npc.move_timer = NPC_MOVE_COOLDOWN;
    }
}

pub fn update_npc_direction_indicator(
    npc_query: Query<(&Transform, &Facing), (With<Npc>, Without<NpcDirectionIndicator>)>,
    mut indicator_query: Query<(&mut Transform, &mut Sprite), (With<NpcDirectionIndicator>, Without<Npc>)>,
) {
    let Ok((npc_transform, facing)) = npc_query.get_single() else { return };
    let Ok((mut transform, mut sprite)) = indicator_query.get_single_mut() else { return };

    let half = TILE_SIZE * 0.35;
    let bar_half = TILE_SIZE * 0.075;
    let (offset_x, offset_y, width, height) = match facing.0 {
        Direction::Up => (0.0, half - bar_half, TILE_SIZE * 0.7, TILE_SIZE * 0.15),
        Direction::Down => (0.0, -(half - bar_half), TILE_SIZE * 0.7, TILE_SIZE * 0.15),
        Direction::Left => (-(half - bar_half), 0.0, TILE_SIZE * 0.15, TILE_SIZE * 0.7),
        Direction::Right => (half - bar_half, 0.0, TILE_SIZE * 0.15, TILE_SIZE * 0.7),
    };
    transform.translation = Vec3::new(
        npc_transform.translation.x + offset_x,
        npc_transform.translation.y + offset_y,
        0.05,
    );
    sprite.custom_size = Some(Vec2::new(width, height));
}

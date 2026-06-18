use bevy::prelude::*;
use crate::components::{
    Carrying, ConveyorBelt, Direction, Facing, FloorTimer, GameEntity, GridPos, Item,
    ItemKind, Npc, NpcDirectionIndicator, NpcState, Player, Solid, Station, StationKind,
    TableMarker,
};
use crate::level::{grid_to_world, spawn_item_entity, TILE_SIZE, MAP_WIDTH, MAP_HEIGHT};
use crate::resources::ShiftState;

const NPC_Z: f32 = 0.02;

pub fn spawn_npc(
    commands: &mut Commands,
    pos: GridPos,
    body_color: Color,
    indicator_color: Color,
    facing: Direction,
    state: NpcState,
    move_cooldown: f32,
    action_cooldown: f32,
) {
    let world_pos = grid_to_world(pos);

    let npc_entity = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: body_color,
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
            Facing(facing),
            Carrying::empty(),
            Npc {
                state,
                move_timer: 0.0,
                action_timer: 0.0,
                move_cooldown,
                action_cooldown,
            },
            GameEntity,
        ))
        .id();

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: indicator_color,
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
        NpcDirectionIndicator { npc_entity },
        GameEntity,
    ));
}

pub fn npc_ai(
    time: Res<Time>,
    mut shift: ResMut<ShiftState>,
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

        npc.state = match npc.state {
            NpcState::WaitingAtConveyor => handle_waiting_at_conveyor(
                &mut npc, &mut facing, &item_on_ground_query,
            ),
            NpcState::PickingUp => handle_picking_up(
                &mut npc, &pos, &mut facing, &mut carrying, &item_on_ground_query, &mut commands,
            ),
            NpcState::MovingToFormer => handle_movement(
                &mut pos, &mut transform, &mut facing, &mut npc,
                GridPos { x: 4, y: 3 }, NpcState::InsertingToFormer, None,
                &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
            ),
            NpcState::InsertingToFormer => handle_inserting_to_former(
                &mut npc, &pos, &mut facing, &mut carrying, &mut station_query, &mut commands,
            ),
            NpcState::WaitingForFormer => handle_waiting_for_station(
                &mut npc, &mut station_query,
                GridPos { x: 3, y: 3 }, StationKind::Former, NpcState::CollectingFromFormer,
            ),
            NpcState::CollectingFromFormer => handle_collect_from_station(
                &mut npc, &pos, &mut facing, &mut carrying, &mut station_query,
                &transform, &mut commands,
                Direction::Left, (-1, 0), StationKind::Former, NpcState::InsertingToOven,
            ),
            NpcState::InsertingToOven => handle_inserting_to_oven(
                &mut npc, &pos, &mut facing, &mut carrying, &mut station_query, &mut commands,
            ),
            NpcState::ReturningToConveyor => handle_movement(
                &mut pos, &mut transform, &mut facing, &mut npc,
                GridPos { x: 4, y: 4 }, NpcState::WaitingAtConveyor, Some(Direction::Left),
                &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
            ),
            NpcState::WaitingAtOven => handle_waiting_for_station(
                &mut npc, &mut station_query,
                GridPos { x: 4, y: 2 }, StationKind::Oven, NpcState::CollectingFromOven,
            ),
            NpcState::CollectingFromOven => handle_collect_from_station(
                &mut npc, &pos, &mut facing, &mut carrying, &mut station_query,
                &transform, &mut commands,
                Direction::Left, (-1, 0), StationKind::Oven, NpcState::MovingToPacker,
            ),
            NpcState::MovingToPacker => handle_movement(
                &mut pos, &mut transform, &mut facing, &mut npc,
                GridPos { x: 8, y: 4 }, NpcState::InsertingToPacker, None,
                &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
            ),
            NpcState::InsertingToPacker => handle_inserting_to_packer(
                &mut npc, &pos, &mut facing, &mut carrying, &mut station_query, &mut commands,
            ),
            NpcState::ReturningToOvenWait => handle_movement(
                &mut pos, &mut transform, &mut facing, &mut npc,
                GridPos { x: 5, y: 2 }, NpcState::WaitingAtOven, Some(Direction::Left),
                &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
            ),
            NpcState::WaitingAtPacker => handle_waiting_for_station(
                &mut npc, &mut station_query,
                GridPos { x: 8, y: 5 }, StationKind::Packer, NpcState::CollectingFromPacker,
            ),
            NpcState::CollectingFromPacker => handle_collect_from_station(
                &mut npc, &pos, &mut facing, &mut carrying, &mut station_query,
                &transform, &mut commands,
                Direction::Right, (1, 0), StationKind::Packer, NpcState::MovingToPalletizer,
            ),
            NpcState::MovingToPalletizer => handle_movement(
                &mut pos, &mut transform, &mut facing, &mut npc,
                GridPos { x: 7, y: 1 }, NpcState::InsertingToPalletizer, None,
                &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
            ),
            NpcState::InsertingToPalletizer => handle_inserting_to_palletizer(
                &mut npc, &pos, &mut facing, &mut carrying, &mut shift, &mut station_query, &mut commands,
            ),
            NpcState::ReturningToPackerWait => handle_movement(
                &mut pos, &mut transform, &mut facing, &mut npc,
                GridPos { x: 7, y: 5 }, NpcState::WaitingAtPacker, Some(Direction::Right),
                &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
            ),
        }
        .unwrap_or(npc.state);
    }
}

fn handle_waiting_at_conveyor(
    npc: &mut Npc,
    facing: &mut Facing,
    item_on_ground_query: &Query<(Entity, &Item, &GridPos), (Without<Npc>, Without<Player>)>,
) -> Option<NpcState> {
    if npc.action_timer > 0.0 {
        return None;
    }
    let has_product = item_on_ground_query
        .iter()
        .any(|(_, item, ip)| *ip == GridPos { x: 3, y: 4 } && item.kind == ItemKind::DoughBatch);
    if has_product {
        facing.0 = Direction::Left;
        Some(NpcState::PickingUp)
    } else {
        npc.action_timer = npc.action_cooldown;
        None
    }
}

fn handle_picking_up(
    npc: &mut Npc,
    pos: &GridPos,
    facing: &mut Facing,
    carrying: &mut Carrying,
    item_on_ground_query: &Query<(Entity, &Item, &GridPos), (Without<Npc>, Without<Player>)>,
    commands: &mut Commands,
) -> Option<NpcState> {
    if npc.action_timer > 0.0 {
        return None;
    }
    facing.0 = Direction::Left;
    let front_pos = GridPos { x: pos.x - 1, y: pos.y };
    for (item_entity, item, ip) in item_on_ground_query.iter() {
        if *ip == front_pos && item.kind == ItemKind::DoughBatch {
            if carrying.0.is_none() {
                commands.entity(item_entity).remove::<GridPos>();
                commands.entity(item_entity).remove::<FloorTimer>();
                carrying.0 = Some((item_entity, item.kind));
                return Some(NpcState::MovingToFormer);
            }
        }
    }
    npc.action_timer = npc.action_cooldown;
    Some(NpcState::WaitingAtConveyor)
}

fn handle_movement(
    pos: &mut GridPos,
    transform: &mut Transform,
    facing: &mut Facing,
    npc: &mut Npc,
    target: GridPos,
    on_arrival: NpcState,
    arrival_facing: Option<Direction>,
    solid_query: &Query<&GridPos, (With<Solid>, Without<Item>, Without<Npc>)>,
    station_pos_query: &Query<&GridPos, (With<Station>, Without<Npc>)>,
    conveyor_pos_query: &Query<&GridPos, (With<ConveyorBelt>, Without<Npc>)>,
    player_query: &Query<&GridPos, (With<Player>, Without<Npc>)>,
) -> Option<NpcState> {
    if move_npc_toward(pos, transform, target, solid_query, station_pos_query, conveyor_pos_query, player_query, npc) {
        if let Some(dir) = arrival_facing {
            facing.0 = dir;
        }
        Some(on_arrival)
    } else {
        None
    }
}

fn handle_inserting_to_former(
    npc: &mut Npc,
    pos: &GridPos,
    facing: &mut Facing,
    carrying: &mut Carrying,
    station_query: &mut Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    commands: &mut Commands,
) -> Option<NpcState> {
    if npc.action_timer > 0.0 {
        return None;
    }
    facing.0 = Direction::Left;
    let front_pos = GridPos { x: pos.x - 1, y: pos.y };
    for (_, mut station, station_pos) in station_query.iter_mut() {
        if *station_pos == front_pos && station.kind == StationKind::Former {
            if let Some((_, carried_kind)) = &carrying.0 {
                if *carried_kind == ItemKind::DoughBatch && !station.busy && !station.has_output {
                    carrying.clear(commands);
                    station.busy = true;
                    station.timer = 0.0;
                    return Some(NpcState::WaitingForFormer);
                }
            }
            break;
        }
    }
    npc.action_timer = npc.action_cooldown;
    None
}

fn handle_waiting_for_station(
    npc: &mut Npc,
    station_query: &mut Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    target_pos: GridPos,
    kind: StationKind,
    next_state: NpcState,
) -> Option<NpcState> {
    if npc.action_timer > 0.0 {
        return None;
    }
    npc.action_timer = npc.action_cooldown;
    if station_query
        .iter()
        .any(|(_, s, sp)| *sp == target_pos && s.kind == kind && s.has_output && !s.busy)
    {
        Some(next_state)
    } else {
        None
    }
}

fn handle_collect_from_station(
    npc: &mut Npc,
    pos: &GridPos,
    facing: &mut Facing,
    carrying: &mut Carrying,
    station_query: &mut Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    transform: &Transform,
    commands: &mut Commands,
    facing_dir: Direction,
    front_delta: (i32, i32),
    expected_kind: StationKind,
    next_state: NpcState,
) -> Option<NpcState> {
    if npc.action_timer > 0.0 {
        return None;
    }
    facing.0 = facing_dir;
    let front_pos = GridPos { x: pos.x + front_delta.0, y: pos.y + front_delta.1 };
    for (_, mut station, station_pos) in station_query.iter_mut() {
        if *station_pos == front_pos && station.kind == expected_kind {
            if station.has_output && carrying.0.is_none() {
                let item_entity = spawn_item_entity(
                    commands,
                    station.output_kind,
                    Vec3::new(transform.translation.x, transform.translation.y, 0.1),
                );
                carrying.0 = Some((item_entity, station.output_kind));
                station.has_output = false;
                return Some(next_state);
            }
            break;
        }
    }
    npc.action_timer = npc.action_cooldown;
    None
}

fn handle_inserting_to_oven(
    npc: &mut Npc,
    pos: &GridPos,
    facing: &mut Facing,
    carrying: &mut Carrying,
    station_query: &mut Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    commands: &mut Commands,
) -> Option<NpcState> {
    if npc.action_timer > 0.0 {
        return None;
    }
    facing.0 = Direction::Down;
    let front_pos = GridPos { x: pos.x, y: pos.y - 1 };
    for (_, mut station, station_pos) in station_query.iter_mut() {
        if *station_pos == front_pos && station.kind == StationKind::Oven {
            if carrying.0.as_ref().map(|(_, k)| *k) == Some(ItemKind::RawCrustTray)
                && !station.busy
                && !station.has_output
            {
                carrying.clear(commands);
                station.busy = true;
                station.timer = 0.0;
                return Some(NpcState::ReturningToConveyor);
            }
            break;
        }
    }
    npc.action_timer = npc.action_cooldown;
    None
}

fn handle_inserting_to_packer(
    npc: &mut Npc,
    pos: &GridPos,
    facing: &mut Facing,
    carrying: &mut Carrying,
    station_query: &mut Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    commands: &mut Commands,
) -> Option<NpcState> {
    if npc.action_timer > 0.0 {
        return None;
    }
    facing.0 = Direction::Up;
    let front_pos = GridPos { x: pos.x, y: pos.y + 1 };
    for (_, mut station, station_pos) in station_query.iter_mut() {
        if *station_pos == front_pos && station.kind == StationKind::Packer {
            if carrying.0.as_ref().map(|(_, k)| *k) == Some(station.accepted_kind)
                && !station.busy
                && !station.has_output
            {
                carrying.clear(commands);
                station.packer_count += 1;
                if station.packer_count >= 3 {
                    station.busy = true;
                    station.timer = 0.0;
                    station.packer_count = 0;
                }
                return Some(NpcState::ReturningToOvenWait);
            }
            break;
        }
    }
    npc.action_timer = npc.action_cooldown;
    None
}

fn handle_inserting_to_palletizer(
    npc: &mut Npc,
    pos: &GridPos,
    facing: &mut Facing,
    carrying: &mut Carrying,
    shift: &mut ShiftState,
    station_query: &mut Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    commands: &mut Commands,
) -> Option<NpcState> {
    if npc.action_timer > 0.0 {
        return None;
    }
    facing.0 = Direction::Left;
    let front_pos = GridPos { x: pos.x - 1, y: pos.y };
    for (_, station, station_pos) in station_query.iter_mut() {
        if *station_pos == front_pos && station.kind == StationKind::Palletizer {
            if carrying.0.as_ref().map(|(_, k)| *k) == Some(ItemKind::Case) {
                carrying.clear(commands);
                shift.cases_completed += 1;
                return Some(NpcState::ReturningToPackerWait);
            }
            break;
        }
    }
    npc.action_timer = npc.action_cooldown;
    None
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
        npc.move_timer = npc.move_cooldown;
    }
}

fn move_npc_toward(
    pos: &mut GridPos,
    transform: &mut Transform,
    target: GridPos,
    solid_query: &Query<&GridPos, (With<Solid>, Without<Item>, Without<Npc>)>,
    station_pos_query: &Query<&GridPos, (With<Station>, Without<Npc>)>,
    conveyor_pos_query: &Query<&GridPos, (With<ConveyorBelt>, Without<Npc>)>,
    player_query: &Query<&GridPos, (With<Player>, Without<Npc>)>,
    npc: &mut Npc,
) -> bool {
    try_npc_move(pos, transform, target, solid_query, station_pos_query, conveyor_pos_query, player_query, npc);
    pos.x == target.x && pos.y == target.y
}

pub fn update_npc_direction_indicator(
    npc_query: Query<(Entity, &Transform, &Facing), (With<Npc>, Without<NpcDirectionIndicator>)>,
    mut indicator_query: Query<(&NpcDirectionIndicator, &mut Transform, &mut Sprite)>,
) {
    for (npc_entity, npc_transform, facing) in npc_query.iter() {
        for (indicator, mut transform, mut sprite) in indicator_query.iter_mut() {
            if indicator.npc_entity != npc_entity {
                continue;
            }

            let (offset_x, offset_y, width, height) = facing.0.indicator_offset(
                TILE_SIZE * 0.35,
                TILE_SIZE * 0.075,
            );
            transform.translation = Vec3::new(
                npc_transform.translation.x + offset_x,
                npc_transform.translation.y + offset_y,
                0.05,
            );
            sprite.custom_size = Some(Vec2::new(width, height));
        }
    }
}

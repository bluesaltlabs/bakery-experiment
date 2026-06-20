use bevy::prelude::*;
use crate::components::{
    Carrying, ConveyorBelt, ConveyorLoaderState, ConveyorLoaderTargets, Direction, Facing,
    FloorTimer, GridPos, Item, ItemKind, Npc, Player, Solid, Station, StationKind, TableMarker,
};
use super::movement;

fn handle_waiting_at_conveyor(
    npc: &mut Npc,
    facing: &mut Facing,
    targets: &ConveyorLoaderTargets,
    item_on_ground_query: &Query<(Entity, &Item, &GridPos), (Without<Npc>, Without<Player>)>,
) -> Option<ConveyorLoaderState> {
    if npc.action_timer > 0.0 {
        return None;
    }
    let check_pos = targets.item_check();
    let has_product = item_on_ground_query
        .iter()
        .any(|(_, item, ip)| *ip == check_pos && item.kind == ItemKind::DoughBatch);
    if has_product {
        facing.0 = Direction::Left;
        Some(ConveyorLoaderState::PickingUp)
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
    targets: &ConveyorLoaderTargets,
    item_on_ground_query: &Query<(Entity, &Item, &GridPos), (Without<Npc>, Without<Player>)>,
    commands: &mut Commands,
) -> Option<ConveyorLoaderState> {
    if npc.action_timer > 0.0 {
        return None;
    }
    facing.0 = Direction::Left;
    let check_pos = targets.item_check();
    let front_pos = GridPos { x: pos.x - 1, y: pos.y };
    for (item_entity, item, ip) in item_on_ground_query.iter() {
        if *ip == front_pos && *ip == check_pos && item.kind == ItemKind::DoughBatch {
            if carrying.0.is_none() {
                commands.entity(item_entity).remove::<GridPos>();
                commands.entity(item_entity).remove::<FloorTimer>();
                carrying.0 = Some((item_entity, item.kind));
                return Some(ConveyorLoaderState::MovingToFormer);
            }
        }
    }
    npc.action_timer = npc.action_cooldown;
    Some(ConveyorLoaderState::WaitingAtConveyor)
}

fn handle_moving(
    pos: &mut GridPos,
    transform: &mut Transform,
    facing: &mut Facing,
    npc: &mut Npc,
    target: GridPos,
    on_arrival: ConveyorLoaderState,
    arrival_facing: Option<Direction>,
    solid_query: &Query<&GridPos, (With<Solid>, Without<Item>, Without<Npc>)>,
    station_pos_query: &Query<&GridPos, (With<Station>, Without<Npc>)>,
    conveyor_pos_query: &Query<&GridPos, (With<ConveyorBelt>, Without<Npc>)>,
    player_query: &Query<&GridPos, (With<Player>, Without<Npc>)>,
) -> Option<ConveyorLoaderState> {
    if movement::move_npc_toward(pos, transform, target, solid_query, station_pos_query, conveyor_pos_query, player_query, npc) {
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
) -> Option<ConveyorLoaderState> {
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
                    return Some(ConveyorLoaderState::WaitingForFormer);
                }
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
) -> Option<ConveyorLoaderState> {
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
                return Some(ConveyorLoaderState::ReturningToConveyor);
            }
            break;
        }
    }
    npc.action_timer = npc.action_cooldown;
    None
}

pub fn conveyor_loader_ai(
    time: Res<Time>,
    mut npc_query: Query<(
        &mut GridPos, &mut Facing, &mut Carrying, &mut Npc,
        &mut ConveyorLoaderState, &mut Transform, &ConveyorLoaderTargets,
    )>,
    mut station_query: Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    item_on_ground_query: Query<(Entity, &Item, &GridPos), (Without<Npc>, Without<Player>)>,
    station_pos_query: Query<&GridPos, (With<Station>, Without<Npc>)>,
    conveyor_pos_query: Query<&GridPos, (With<ConveyorBelt>, Without<Npc>)>,
    solid_query: Query<&GridPos, (With<Solid>, Without<Item>, Without<Npc>)>,
    player_query: Query<&GridPos, (With<Player>, Without<Npc>)>,
    mut commands: Commands,
) {
    let dt = time.delta_seconds();

    for (mut pos, mut facing, mut carrying, mut npc, mut state, mut transform, targets) in npc_query.iter_mut() {
        npc.move_timer -= dt;
        npc.action_timer -= dt;

        *state = match *state {
            ConveyorLoaderState::WaitingAtConveyor => handle_waiting_at_conveyor(
                &mut npc, &mut facing, targets, &item_on_ground_query,
            ),
            ConveyorLoaderState::PickingUp => handle_picking_up(
                &mut npc, &pos, &mut facing, &mut carrying, targets, &item_on_ground_query, &mut commands,
            ),
            ConveyorLoaderState::MovingToFormer => {
                let target = targets.operate_pos();
                handle_moving(
                    &mut pos, &mut transform, &mut facing, &mut npc,
                    target, ConveyorLoaderState::InsertingToFormer, None,
                    &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
                )
            }
            ConveyorLoaderState::InsertingToFormer => handle_inserting_to_former(
                &mut npc, &pos, &mut facing, &mut carrying, &mut station_query, &mut commands,
            ),
            ConveyorLoaderState::WaitingForFormer => {
                let check_pos = targets.former_pos();
                if super::try_wait_for_station_output(
                    &mut npc, &mut station_query,
                    check_pos, StationKind::Former,
                ) {
                    Some(ConveyorLoaderState::CollectingFromFormer)
                } else {
                    None
                }
            }
            ConveyorLoaderState::CollectingFromFormer => {
                if super::try_collect_from_station(
                    &mut npc, &pos, &mut facing, &mut carrying, &mut station_query,
                    &transform, &mut commands,
                    Direction::Left, (-1, 0), StationKind::Former,
                ) {
                    Some(ConveyorLoaderState::InsertingToOven)
                } else {
                    None
                }
            }
            ConveyorLoaderState::InsertingToOven => handle_inserting_to_oven(
                &mut npc, &pos, &mut facing, &mut carrying, &mut station_query, &mut commands,
            ),
            ConveyorLoaderState::ReturningToConveyor => {
                let target = targets.spawn;
                handle_moving(
                    &mut pos, &mut transform, &mut facing, &mut npc,
                    target, ConveyorLoaderState::WaitingAtConveyor, Some(Direction::Left),
                    &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
                )
            }
        }
        .unwrap_or(*state);
    }
}

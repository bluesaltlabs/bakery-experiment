use bevy::prelude::*;
use crate::components::{
    Carrying, ConveyorBelt, Direction, Facing, GridPos, Item, Npc, OvenHaulerState,
    OvenHaulerTargets, Player, Solid, Station, StationKind, TableMarker,
};
use crate::resources::EditorMode;
use super::movement;

fn handle_moving(
    pos: &mut GridPos,
    transform: &mut Transform,
    facing: &mut Facing,
    npc: &mut Npc,
    target: GridPos,
    on_arrival: OvenHaulerState,
    arrival_facing: Option<Direction>,
    solid_query: &Query<&GridPos, (With<Solid>, Without<Item>, Without<Npc>)>,
    station_pos_query: &Query<&GridPos, (With<Station>, Without<Npc>)>,
    conveyor_pos_query: &Query<&GridPos, (With<ConveyorBelt>, Without<Npc>)>,
    player_query: &Query<&GridPos, (With<Player>, Without<Npc>)>,
) -> Option<OvenHaulerState> {
    if movement::move_npc_toward(pos, transform, target, solid_query, station_pos_query, conveyor_pos_query, player_query, npc) {
        if let Some(dir) = arrival_facing {
            facing.0 = dir;
        }
        Some(on_arrival)
    } else {
        None
    }
}

fn handle_inserting_to_packer(
    npc: &mut Npc,
    pos: &GridPos,
    facing: &mut Facing,
    carrying: &mut Carrying,
    station_query: &mut Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    commands: &mut Commands,
) -> Option<OvenHaulerState> {
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
                return Some(OvenHaulerState::ReturningToOvenWait);
            }
            break;
        }
    }
    npc.action_timer = npc.action_cooldown;
    None
}

pub fn oven_hauler_ai(
    editor: Res<EditorMode>,
    time: Res<Time>,
    mut npc_query: Query<(
        &mut GridPos, &mut Facing, &mut Carrying, &mut Npc,
        &mut OvenHaulerState, &mut Transform, &OvenHaulerTargets,
    )>,
    mut station_query: Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    station_pos_query: Query<&GridPos, (With<Station>, Without<Npc>)>,
    conveyor_pos_query: Query<&GridPos, (With<ConveyorBelt>, Without<Npc>)>,
    solid_query: Query<&GridPos, (With<Solid>, Without<Item>, Without<Npc>)>,
    player_query: Query<&GridPos, (With<Player>, Without<Npc>)>,
    mut commands: Commands,
) {
    if editor.0 {
        return;
    }
    let dt = time.delta_seconds();

    for (mut pos, mut facing, mut carrying, mut npc, mut state, mut transform, targets) in npc_query.iter_mut() {
        npc.move_timer -= dt;
        npc.action_timer -= dt;

        *state = match *state {
            OvenHaulerState::WaitingAtOven => {
                let check_pos = targets.oven_pos();
                if super::try_wait_for_station_output(
                    &mut npc, &mut station_query,
                    check_pos, StationKind::Oven,
                ) {
                    Some(OvenHaulerState::CollectingFromOven)
                } else {
                    None
                }
            }
            OvenHaulerState::CollectingFromOven => {
                if super::try_collect_from_station(
                    &mut npc, &pos, &mut facing, &mut carrying, &mut station_query,
                    &transform, &mut commands,
                    Direction::Left, (-1, 0), StationKind::Oven,
                ) {
                    Some(OvenHaulerState::MovingToPacker)
                } else {
                    None
                }
            }
            OvenHaulerState::MovingToPacker => {
                let target = targets.packer_stand();
                handle_moving(
                    &mut pos, &mut transform, &mut facing, &mut npc,
                    target, OvenHaulerState::InsertingToPacker, None,
                    &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
                )
            }
            OvenHaulerState::InsertingToPacker => handle_inserting_to_packer(
                &mut npc, &pos, &mut facing, &mut carrying, &mut station_query, &mut commands,
            ),
            OvenHaulerState::ReturningToOvenWait => {
                let target = targets.spawn;
                handle_moving(
                    &mut pos, &mut transform, &mut facing, &mut npc,
                    target, OvenHaulerState::WaitingAtOven, Some(Direction::Left),
                    &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
                )
            }
        }
        .unwrap_or(*state);
    }
}

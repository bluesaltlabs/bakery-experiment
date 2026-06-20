use bevy::prelude::*;
use crate::components::{
    Carrying, ConveyorBelt, Direction, Facing, GridPos, Item, ItemKind, Npc,
    PackerHaulerState, PackerHaulerTargets, Player, Solid, Station, StationKind, TableMarker,
};
use crate::resources::ShiftState;
use super::movement;

fn handle_moving(
    pos: &mut GridPos,
    transform: &mut Transform,
    facing: &mut Facing,
    npc: &mut Npc,
    target: GridPos,
    on_arrival: PackerHaulerState,
    arrival_facing: Option<Direction>,
    solid_query: &Query<&GridPos, (With<Solid>, Without<Item>, Without<Npc>)>,
    station_pos_query: &Query<&GridPos, (With<Station>, Without<Npc>)>,
    conveyor_pos_query: &Query<&GridPos, (With<ConveyorBelt>, Without<Npc>)>,
    player_query: &Query<&GridPos, (With<Player>, Without<Npc>)>,
) -> Option<PackerHaulerState> {
    if movement::move_npc_toward(pos, transform, target, solid_query, station_pos_query, conveyor_pos_query, player_query, npc) {
        if let Some(dir) = arrival_facing {
            facing.0 = dir;
        }
        Some(on_arrival)
    } else {
        None
    }
}

fn handle_inserting_to_palletizer(
    npc: &mut Npc,
    pos: &GridPos,
    facing: &mut Facing,
    carrying: &mut Carrying,
    shift: &mut ShiftState,
    station_query: &mut Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    commands: &mut Commands,
) -> Option<PackerHaulerState> {
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
                return Some(PackerHaulerState::ReturningToPackerWait);
            }
            break;
        }
    }
    npc.action_timer = npc.action_cooldown;
    None
}

pub fn packer_hauler_ai(
    time: Res<Time>,
    mut shift: ResMut<ShiftState>,
    mut npc_query: Query<(
        &mut GridPos, &mut Facing, &mut Carrying, &mut Npc,
        &mut PackerHaulerState, &mut Transform, &PackerHaulerTargets,
    )>,
    mut station_query: Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
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

    for (mut pos, mut facing, mut carrying, mut npc, mut state, mut transform, targets) in npc_query.iter_mut() {
        npc.move_timer -= dt;
        npc.action_timer -= dt;

        *state = match *state {
            PackerHaulerState::WaitingAtPacker => {
                let check_pos = targets.packer_pos();
                if super::try_wait_for_station_output(
                    &mut npc, &mut station_query,
                    check_pos, StationKind::Packer,
                ) {
                    Some(PackerHaulerState::CollectingFromPacker)
                } else {
                    None
                }
            }
            PackerHaulerState::CollectingFromPacker => {
                if super::try_collect_from_station(
                    &mut npc, &pos, &mut facing, &mut carrying, &mut station_query,
                    &transform, &mut commands,
                    Direction::Right, (1, 0), StationKind::Packer,
                ) {
                    Some(PackerHaulerState::MovingToPalletizer)
                } else {
                    None
                }
            }
            PackerHaulerState::MovingToPalletizer => {
                let target = targets.palletizer_stand();
                handle_moving(
                    &mut pos, &mut transform, &mut facing, &mut npc,
                    target, PackerHaulerState::InsertingToPalletizer, None,
                    &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
                )
            }
            PackerHaulerState::InsertingToPalletizer => handle_inserting_to_palletizer(
                &mut npc, &pos, &mut facing, &mut carrying, &mut shift, &mut station_query, &mut commands,
            ),
            PackerHaulerState::ReturningToPackerWait => {
                let target = targets.spawn;
                handle_moving(
                    &mut pos, &mut transform, &mut facing, &mut npc,
                    target, PackerHaulerState::WaitingAtPacker, Some(Direction::Right),
                    &solid_query, &station_pos_query, &conveyor_pos_query, &player_query,
                )
            }
        }
        .unwrap_or(*state);
    }
}

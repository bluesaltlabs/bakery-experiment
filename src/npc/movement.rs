use bevy::prelude::*;
use crate::components::{ConveyorBelt, GridPos, Item, Npc, Player, Solid, Station};
use crate::level::{grid_to_world, MAP_WIDTH, MAP_HEIGHT};

pub(crate) const NPC_Z: f32 = 0.02;

pub(crate) fn is_tile_blocked_for_npc(
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

pub(crate) fn try_npc_move(
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

pub(crate) fn move_npc_toward(
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

mod movement;
pub mod conveyor_loader;
pub mod oven_hauler;
pub mod packer_hauler;

use bevy::prelude::*;
use crate::agent;
use crate::components::{
    Carrying, ConveyorLoaderState, ConveyorLoaderTargets, Direction, Facing, GameEntity,
    GridPos, Npc, NpcKind, OvenHaulerState, OvenHaulerTargets,
    PackerHaulerState, PackerHaulerTargets, Station, StationKind, TableMarker,
};
use crate::level::{grid_to_world, TILE_SIZE, Z_NPC};
use crate::station_config::StationConfig;

pub fn spawn_npc(
    commands: &mut Commands,
    pos: GridPos,
    body_color: Color,
    indicator_color: Color,
    facing: Direction,
    move_cooldown: f32,
    action_cooldown: f32,
) -> Entity {
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
                    Z_NPC,
                )),
                ..default()
            },
            agent::AgentBundle {
                grid_pos: pos,
                facing: Facing(facing),
                carrying: Carrying::empty(),
                game_entity: GameEntity,
            },
            Npc {
                move_timer: 0.0,
                action_timer: 0.0,
                move_cooldown,
                action_cooldown,
            },
        ))
        .id();

    agent::spawn_indicator(commands, npc_entity, world_pos, indicator_color);

    npc_entity
}

pub fn spawn_conveyor_loader(
    commands: &mut Commands,
    pos: GridPos,
    body_color: Color,
    indicator_color: Color,
    facing: Direction,
    move_cooldown: f32,
    action_cooldown: f32,
) {
    let entity = spawn_npc(commands, pos, body_color, indicator_color, facing, move_cooldown, action_cooldown);
    commands.entity(entity).insert(ConveyorLoaderState::WaitingAtConveyor);
    commands.entity(entity).insert(ConveyorLoaderTargets::new(pos));
}

pub fn spawn_oven_hauler(
    commands: &mut Commands,
    pos: GridPos,
    body_color: Color,
    indicator_color: Color,
    facing: Direction,
    move_cooldown: f32,
    action_cooldown: f32,
) {
    let entity = spawn_npc(commands, pos, body_color, indicator_color, facing, move_cooldown, action_cooldown);
    commands.entity(entity).insert(OvenHaulerState::WaitingAtOven);
    commands.entity(entity).insert(OvenHaulerTargets::new(pos));
}

pub fn spawn_packer_hauler(
    commands: &mut Commands,
    pos: GridPos,
    body_color: Color,
    indicator_color: Color,
    facing: Direction,
    move_cooldown: f32,
    action_cooldown: f32,
) {
    let entity = spawn_npc(commands, pos, body_color, indicator_color, facing, move_cooldown, action_cooldown);
    commands.entity(entity).insert(PackerHaulerState::WaitingAtPacker);
    commands.entity(entity).insert(PackerHaulerTargets::new(pos));
}

pub fn spawn_npc_from_data(commands: &mut Commands, data: &crate::resources::NpcSpawnData) {
    let (body_color, indicator_color, move_cooldown, action_cooldown) = match data.kind {
        NpcKind::ConveyorLoader => (
            Color::srgb(1.0, 0.5, 0.0),
            Color::srgb(1.0, 0.7, 0.3),
            1.0, 0.5,
        ),
        NpcKind::OvenHauler => (
            Color::srgb(0.2, 0.8, 0.5),
            Color::srgb(0.4, 1.0, 0.6),
            0.5, 0.25,
        ),
        NpcKind::PackerHauler => (
            Color::srgb(0.3, 0.5, 0.9),
            Color::srgb(0.5, 0.7, 1.0),
            0.5, 0.25,
        ),
    };
    match data.kind {
        NpcKind::ConveyorLoader => {
            spawn_conveyor_loader(commands, data.pos, body_color, indicator_color, data.facing, move_cooldown, action_cooldown);
        }
        NpcKind::OvenHauler => {
            spawn_oven_hauler(commands, data.pos, body_color, indicator_color, data.facing, move_cooldown, action_cooldown);
        }
        NpcKind::PackerHauler => {
            spawn_packer_hauler(commands, data.pos, body_color, indicator_color, data.facing, move_cooldown, action_cooldown);
        }
    }
}

pub(crate) fn try_wait_for_station_output(
    npc: &mut Npc,
    station_query: &mut Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Npc>)>,
    target_pos: GridPos,
    kind: StationKind,
) -> bool {
    if npc.action_timer > 0.0 {
        return false;
    }
    npc.action_timer = npc.action_cooldown;
    station_query
        .iter()
        .any(|(_, s, sp)| *sp == target_pos && s.kind == kind && s.has_output && !s.busy)
}

pub(crate) fn try_collect_from_station(
    config: &StationConfig,
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
) -> bool {
    if npc.action_timer > 0.0 {
        return false;
    }
    facing.0 = facing_dir;
    let front_pos = GridPos { x: pos.x + front_delta.0, y: pos.y + front_delta.1 };
    let def = config.def(expected_kind);
    for (_, mut station, station_pos) in station_query.iter_mut() {
        if *station_pos == front_pos && station.kind == expected_kind {
            if station.has_output && carrying.0.is_none() {
                let item_entity = crate::level::spawn_item_entity(
                    commands,
                    def.output_kind,
                    Vec3::new(transform.translation.x, transform.translation.y, 0.1),
                );
                carrying.0 = Some((item_entity, def.output_kind));
                station.has_output = false;
                return true;
            }
            break;
        }
    }
    npc.action_timer = npc.action_cooldown;
    false
}

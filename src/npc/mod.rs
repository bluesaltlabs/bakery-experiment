mod movement;
pub mod conveyor_loader;
pub mod oven_hauler;
pub mod packer_hauler;

use movement::*;
use bevy::prelude::*;
use crate::components::{
    Carrying, ConveyorLoaderState, Direction, Facing, GameEntity, GridPos, Npc,
    NpcDirectionIndicator, OvenHaulerState, PackerHaulerState, Station, StationKind,
    TableMarker,
};
use crate::level::{grid_to_world, spawn_item_entity, TILE_SIZE};

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
                    NPC_Z,
                )),
                ..default()
            },
            pos,
            Facing(facing),
            Carrying::empty(),
            Npc {
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
                return true;
            }
            break;
        }
    }
    npc.action_timer = npc.action_cooldown;
    false
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

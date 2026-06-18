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
                    npc.action_timer = npc.action_cooldown;
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
                    if carrying.0.is_none() {
                        commands.entity(item_entity).remove::<GridPos>();
                        commands.entity(item_entity).remove::<FloorTimer>();
                        carrying.0 = Some((item_entity, item_kind));
                        picked_up = true;
                    }
                }

                if picked_up {
                    npc.state = NpcState::MovingToFormer;
                } else {
                    npc.action_timer = npc.action_cooldown;
                    npc.state = NpcState::WaitingAtConveyor;
                }
            }

            NpcState::MovingToFormer => {
                if move_npc_toward(&mut pos, &mut transform, GridPos { x: 4, y: 3 }, &solid_query, &station_pos_query, &conveyor_pos_query, &player_query, &mut npc) {
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
                for (_, mut station, station_pos) in station_query.iter_mut() {
                    if *station_pos == front_pos && station.kind == StationKind::Former {
                        if let Some((carried_entity, carried_kind)) = carrying.0 {
                            if carried_kind == ItemKind::DoughBatch
                                && !station.busy
                                && !station.has_output
                            {
                                commands.entity(carried_entity).despawn();
                                carrying.0 = None;
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
                    npc.action_timer = npc.action_cooldown;
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

                npc.action_timer = npc.action_cooldown;
            }

            NpcState::CollectingFromFormer => {
                if npc.action_timer > 0.0 {
                    continue;
                }
                facing.0 = Direction::Left;
                let front_pos = GridPos { x: pos.x - 1, y: pos.y };

                let mut collected = false;
                for (_, mut station, station_pos) in station_query.iter_mut() {
                    if *station_pos == front_pos && station.kind == StationKind::Former {
                        if station.has_output && carrying.0.is_none() {
                            let item_entity = spawn_item_entity(
                                &mut commands,
                                station.output_kind,
                                Vec3::new(transform.translation.x, transform.translation.y, 0.1),
                            );
                            carrying.0 = Some((item_entity, station.output_kind));
                            station.has_output = false;
                            collected = true;
                            npc.state = NpcState::InsertingToOven;
                        }
                        break;
                    }
                }

                if !collected {
                    npc.action_timer = npc.action_cooldown;
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
                for (_, mut station, station_pos) in station_query.iter_mut() {
                    if *station_pos == front_pos && station.kind == StationKind::Oven {
                        if carrying.0.map(|(_, k)| k) == Some(ItemKind::RawCrustTray)
                            && !station.busy
                            && !station.has_output
                        {
                            if let Some((carried_entity, _)) = carrying.0 {
                                commands.entity(carried_entity).despawn();
                                carrying.0 = None;
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
                    npc.action_timer = npc.action_cooldown;
                }
            }

            NpcState::ReturningToConveyor => {
                if move_npc_toward(&mut pos, &mut transform, GridPos { x: 4, y: 4 }, &solid_query, &station_pos_query, &conveyor_pos_query, &player_query, &mut npc) {
                    facing.0 = Direction::Left;
                    npc.state = NpcState::WaitingAtConveyor;
                }
            }

            NpcState::WaitingAtOven => {
                if npc.action_timer > 0.0 {
                    continue;
                }

                let has_output = station_query
                    .iter()
                    .any(|(_, s, sp)| *sp == GridPos { x: 4, y: 2 } && s.kind == StationKind::Oven && s.has_output && !s.busy);

                if has_output {
                    npc.state = NpcState::CollectingFromOven;
                } else {
                    npc.action_timer = npc.action_cooldown;
                }
            }

            NpcState::CollectingFromOven => {
                if npc.action_timer > 0.0 {
                    continue;
                }
                facing.0 = Direction::Left;
                let front_pos = GridPos { x: pos.x - 1, y: pos.y };

                let mut collected = false;
                for (_, mut station, station_pos) in station_query.iter_mut() {
                    if *station_pos == front_pos && station.kind == StationKind::Oven {
                        if station.has_output && carrying.0.is_none() {
                            let item_entity = spawn_item_entity(
                                &mut commands,
                                station.output_kind,
                                Vec3::new(transform.translation.x, transform.translation.y, 0.1),
                            );
                            carrying.0 = Some((item_entity, station.output_kind));
                            station.has_output = false;
                            collected = true;
                            npc.state = NpcState::MovingToPacker;
                        }
                        break;
                    }
                }

                if !collected {
                    npc.action_timer = npc.action_cooldown;
                }
            }

            NpcState::MovingToPacker => {
                if move_npc_toward(&mut pos, &mut transform, GridPos { x: 8, y: 4 }, &solid_query, &station_pos_query, &conveyor_pos_query, &player_query, &mut npc) {
                    npc.state = NpcState::InsertingToPacker;
                }
            }

            NpcState::InsertingToPacker => {
                if npc.action_timer > 0.0 {
                    continue;
                }
                facing.0 = Direction::Up;
                let front_pos = GridPos { x: pos.x, y: pos.y + 1 };

                let mut inserted = false;
                for (_, mut station, station_pos) in station_query.iter_mut() {
                    if *station_pos == front_pos && station.kind == StationKind::Packer {
                        if carrying.0.map(|(_, k)| k) == Some(station.accepted_kind)
                            && !station.busy
                            && !station.has_output
                        {
                            if let Some((carried_entity, _)) = carrying.0 {
                                commands.entity(carried_entity).despawn();
                                carrying.0 = None;

                                station.packer_count += 1;
                                if station.packer_count >= 3 {
                                    station.busy = true;
                                    station.timer = 0.0;
                                    station.packer_count = 0;
                                }
                                inserted = true;
                            }
                        }
                        break;
                    }
                }

                if inserted {
                    npc.state = NpcState::ReturningToOvenWait;
                } else {
                    npc.action_timer = npc.action_cooldown;
                }
            }

            NpcState::ReturningToOvenWait => {
                if move_npc_toward(&mut pos, &mut transform, GridPos { x: 5, y: 2 }, &solid_query, &station_pos_query, &conveyor_pos_query, &player_query, &mut npc) {
                    facing.0 = Direction::Left;
                    npc.state = NpcState::WaitingAtOven;
                }
            }

            NpcState::WaitingAtPacker => {
                if npc.action_timer > 0.0 {
                    continue;
                }

                let has_output = station_query
                    .iter()
                    .any(|(_, s, sp)| *sp == GridPos { x: 8, y: 5 } && s.kind == StationKind::Packer && s.has_output && !s.busy);

                if has_output {
                    npc.state = NpcState::CollectingFromPacker;
                } else {
                    npc.action_timer = npc.action_cooldown;
                }
            }

            NpcState::CollectingFromPacker => {
                if npc.action_timer > 0.0 {
                    continue;
                }
                facing.0 = Direction::Right;
                let front_pos = GridPos { x: pos.x + 1, y: pos.y };

                let mut collected = false;
                for (_, mut station, station_pos) in station_query.iter_mut() {
                    if *station_pos == front_pos && station.kind == StationKind::Packer {
                        if station.has_output && carrying.0.is_none() {
                            let item_entity = spawn_item_entity(
                                &mut commands,
                                station.output_kind,
                                Vec3::new(transform.translation.x, transform.translation.y, 0.1),
                            );
                            carrying.0 = Some((item_entity, station.output_kind));
                            station.has_output = false;
                            collected = true;
                            npc.state = NpcState::MovingToPalletizer;
                        }
                        break;
                    }
                }

                if !collected {
                    npc.action_timer = npc.action_cooldown;
                }
            }

            NpcState::MovingToPalletizer => {
                if move_npc_toward(&mut pos, &mut transform, GridPos { x: 7, y: 1 }, &solid_query, &station_pos_query, &conveyor_pos_query, &player_query, &mut npc) {
                    npc.state = NpcState::InsertingToPalletizer;
                }
            }

            NpcState::InsertingToPalletizer => {
                if npc.action_timer > 0.0 {
                    continue;
                }
                facing.0 = Direction::Left;
                let front_pos = GridPos { x: pos.x - 1, y: pos.y };

                let mut inserted = false;
                for (_, station, station_pos) in station_query.iter_mut() {
                    if *station_pos == front_pos && station.kind == StationKind::Palletizer {
                        if carrying.0.map(|(_, k)| k) == Some(ItemKind::Case) {
                            if let Some((carried_entity, _)) = carrying.0 {
                                commands.entity(carried_entity).despawn();
                                carrying.0 = None;
                                shift.cases_completed += 1;
                            }
                            inserted = true;
                            npc.state = NpcState::ReturningToPackerWait;
                        }
                        break;
                    }
                }

                if !inserted {
                    npc.action_timer = npc.action_cooldown;
                }
            }

            NpcState::ReturningToPackerWait => {
                if move_npc_toward(&mut pos, &mut transform, GridPos { x: 7, y: 5 }, &solid_query, &station_pos_query, &conveyor_pos_query, &player_query, &mut npc) {
                    facing.0 = Direction::Right;
                    npc.state = NpcState::WaitingAtPacker;
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

use bevy::prelude::*;
use crate::audio::AudioEvent;
use crate::components::*;
use crate::level::spawn_item_entity;
use crate::resources::ShiftState;

pub fn update_carried_items(
    carrier_query: Query<(&Carrying, &Transform), Without<Item>>,
    mut item_query: Query<&mut Transform, With<Item>>,
) {
    for (carrying, carrier_transform) in carrier_query.iter() {
        if let Some((entity, _)) = carrying.0 {
            if let Ok(mut item_transform) = item_query.get_mut(entity) {
                item_transform.translation = Vec3::new(
                    carrier_transform.translation.x,
                    carrier_transform.translation.y,
                    0.1,
                );
            }
        }
    }
}

fn try_table_interaction(
    commands: &mut Commands,
    carrying: &mut Carrying,
    front_pos: GridPos,
    table_query: &Query<(Entity, &Station, &GridPos), (With<TableMarker>, Without<Player>)>,
) -> bool {
    if !table_query.iter().any(|(_, _, tp)| *tp == front_pos) {
        return false;
    }
    if let Some((carried_entity, _)) = carrying.0 {
        commands.entity(carried_entity).insert(front_pos);
        carrying.0 = None;
        return true;
    }
    false
}

fn try_station_deposit(
    commands: &mut Commands,
    shift: &mut ShiftState,
    carrying: &mut Carrying,
    station: &mut Station,
) -> bool {
    let Some((_, carried_kind)) = carrying.0 else { return false };

    if carried_kind == ItemKind::Case && station.kind == StationKind::Palletizer {
        carrying.clear(commands);
        shift.cases_completed += 1;
        return true;
    }

    if station.kind != StationKind::Source
        && carried_kind == station.accepted_kind
        && !station.busy
        && !station.has_output
    {
        carrying.clear(commands);

        if station.kind == StationKind::Packer {
            station.packer_count += 1;
            if station.packer_count >= 3 {
                station.busy = true;
                station.timer = 0.0;
                station.packer_count = 0;
            }
        } else {
            station.busy = true;
            station.timer = 0.0;
        }
        return true;
    }
    false
}

fn try_station_pickup(
    commands: &mut Commands,
    carrying: &mut Carrying,
    player_transform: &Transform,
    station: &mut Station,
) -> bool {
    if !station.has_output || carrying.0.is_some() {
        return false;
    }
    let item_entity = spawn_item_entity(
        commands,
        station.output_kind,
        Vec3::new(player_transform.translation.x, player_transform.translation.y, 0.1),
    );
    carrying.0 = Some((item_entity, station.output_kind));
    station.has_output = false;
    true
}

fn try_ground_pickup(
    commands: &mut Commands,
    carrying: &mut Carrying,
    front_pos: GridPos,
    item_on_ground_query: &Query<(Entity, &Item, &GridPos)>,
) -> bool {
    if carrying.0.is_some() {
        return false;
    }
    for (item_entity, item, ground_pos) in item_on_ground_query.iter() {
        if *ground_pos == front_pos {
            commands.entity(item_entity).remove::<GridPos>();
            commands.entity(item_entity).remove::<FloorTimer>();
            carrying.0 = Some((item_entity, item.kind));
            return true;
        }
    }
    false
}

fn try_ground_drop(
    commands: &mut Commands,
    carrying: &mut Carrying,
    front_pos: GridPos,
    conveyor_query: &Query<&GridPos, (With<ConveyorBelt>, Without<Player>)>,
    solid_query: &Query<&GridPos, (With<Solid>, Without<Player>)>,
    item_on_ground_query: &Query<(Entity, &Item, &GridPos)>,
) -> bool {
    let Some((carried_entity, _)) = carrying.0 else { return false };
    let on_conveyor = conveyor_query.iter().any(|gp| *gp == front_pos);
    let blocked = solid_query.iter().any(|gp| *gp == front_pos)
        || item_on_ground_query.iter().any(|(_, _, gp)| *gp == front_pos);
    if blocked {
        return false;
    }
    commands.entity(carried_entity).insert(front_pos);
    if !on_conveyor {
        commands.entity(carried_entity).insert(FloorTimer(10.0));
    }
    carrying.0 = None;
    true
}

pub fn player_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    mut shift: ResMut<ShiftState>,
    mut player_query: Query<(&GridPos, &Facing, &mut Carrying, &Transform), With<Player>>,
    mut station_query: Query<(Entity, &mut Station, &GridPos), (Without<TableMarker>, Without<Player>)>,
    table_query: Query<(Entity, &Station, &GridPos), (With<TableMarker>, Without<Player>)>,
    item_on_ground_query: Query<(Entity, &Item, &GridPos)>,
    solid_query: Query<&GridPos, (With<Solid>, Without<Player>)>,
    conveyor_query: Query<&GridPos, (With<ConveyorBelt>, Without<Player>)>,
    mut commands: Commands,
    mut audio_queue: ResMut<crate::audio::AudioEventQueue>,
) {
    if !keys.just_pressed(KeyCode::KeyE) && !keys.just_pressed(KeyCode::Space) {
        return;
    }
    if shift.game_over {
        return;
    }

    let (player_pos, facing, mut carrying, player_transform) = player_query.single_mut();
    let delta = facing.0.delta();
    let front_pos = GridPos {
        x: player_pos.x + delta.0,
        y: player_pos.y + delta.1,
    };

    if try_table_interaction(&mut commands, &mut carrying, front_pos, &table_query) {
        audio_queue.0.push(AudioEvent::Drop);
        return;
    }

    for (_, mut station, station_pos) in station_query.iter_mut() {
        if *station_pos != front_pos {
            continue;
        }
        if try_station_deposit(&mut commands, &mut shift, &mut carrying, &mut station) {
            audio_queue.0.push(AudioEvent::StationDeposit);
            return;
        }
        if try_station_pickup(&mut commands, &mut carrying, &player_transform, &mut station) {
            audio_queue.0.push(AudioEvent::Pickup);
            return;
        }
        return;
    }

    if try_ground_pickup(&mut commands, &mut carrying, front_pos, &item_on_ground_query) {
        audio_queue.0.push(AudioEvent::Pickup);
        return;
    }

    if try_ground_drop(&mut commands, &mut carrying, front_pos, &conveyor_query, &solid_query, &item_on_ground_query) {
        audio_queue.0.push(AudioEvent::Drop);
    }
}

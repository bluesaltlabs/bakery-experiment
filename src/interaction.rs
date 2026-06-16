use bevy::prelude::*;
use crate::components::*;
use crate::level::{grid_to_world, TILE_SIZE};
use crate::resources::ShiftState;

pub fn update_carried_items(
    player_query: Query<(&Carrying, &Transform), (With<Player>, Without<Item>)>,
    mut item_query: Query<&mut Transform, (With<Item>, Without<GridPos>, Without<Player>)>,
) {
    for (carrying, player_transform) in player_query.iter() {
        if let Some(entity) = carrying.0 {
            if let Ok(mut item_transform) = item_query.get_mut(entity) {
                item_transform.translation = player_transform.translation;
            }
        }
    }
}

pub fn player_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    mut shift: ResMut<ShiftState>,
    mut player_query: Query<(&GridPos, &Facing, &mut Carrying), With<Player>>,
    mut station_query: Query<(Entity, &mut Station, &GridPos)>,
    item_on_ground_query: Query<(Entity, &Item, &GridPos)>,
    item_query: Query<&Item>,
    solid_query: Query<&GridPos, (With<Solid>, Without<Player>)>,
    mut commands: Commands,
) {
    if !keys.just_pressed(KeyCode::KeyE) && !keys.just_pressed(KeyCode::Space) {
        return;
    }

    if shift.game_over {
        return;
    }

    let (player_pos, facing, mut carrying) = player_query.single_mut();
    let delta = facing.0.delta();
    let front_pos = GridPos {
        x: player_pos.x + delta.0,
        y: player_pos.y + delta.1,
    };

    for (_station_entity, mut station, station_pos) in station_query.iter_mut() {
        if *station_pos != front_pos {
            continue;
        }

        match carrying.0 {
            Some(carried_entity) => {
                if let Ok(item) = item_query.get(carried_entity) {
                    if station.kind == StationKind::Palletizer && item.kind == ItemKind::Case {
                        commands.entity(carried_entity).despawn();
                        carrying.0 = None;
                        shift.cases_completed += 1;
                        return;
                    }

                    if station.kind != StationKind::Source
                        && item.kind == station.accepted_kind
                        && !station.busy
                    {
                        commands.entity(carried_entity).despawn();
                        carrying.0 = None;

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
                        return;
                    }
                }
            }
            None => {
                if station.has_output {
                    let item_entity = commands.spawn((
                        Item { kind: station.output_kind },
                        SpriteBundle {
                            sprite: Sprite {
                                color: station.output_kind.color(),
                                custom_size: Some(Vec2::new(TILE_SIZE * 0.45, TILE_SIZE * 0.45)),
                                ..default()
                            },
                            transform: Transform::from_translation(grid_to_world(front_pos)),
                            ..default()
                        },
                        GameEntity,
                    )).id();
                    carrying.0 = Some(item_entity);
                    station.has_output = false;
                    return;
                }
            }
        }
        return;
    }

    if carrying.0.is_none() {
        for (item_entity, _item, ground_pos) in item_on_ground_query.iter() {
            if *ground_pos == front_pos {
                carrying.0 = Some(item_entity);
                commands.entity(item_entity).remove::<GridPos>();
                return;
            }
        }
    }

    if let Some(carried_entity) = carrying.0 {
        let blocked = solid_query.iter().any(|gp| *gp == front_pos)
            || station_query.iter().any(|(_, _, gp)| *gp == front_pos)
            || item_on_ground_query.iter().any(|(_, _, gp)| *gp == front_pos);

        if !blocked {
            commands.entity(carried_entity).insert(front_pos);
            carrying.0 = None;
        }
    }
}

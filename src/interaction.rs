use bevy::prelude::*;
use crate::components::*;
use crate::level::TILE_SIZE;
use crate::resources::ShiftState;

pub fn update_carried_items(
    player_query: Query<(&Carrying, &Transform), (With<Player>, Without<Item>)>,
    mut item_query: Query<&mut Transform, With<Item>>,
) {
    for (carrying, player_transform) in player_query.iter() {
        if let Some(entity) = carrying.entity {
            if let Ok(mut item_transform) = item_query.get_mut(entity) {
                item_transform.translation = Vec3::new(
                    player_transform.translation.x,
                    player_transform.translation.y,
                    0.1,
                );
            }
        }
    }
}

pub fn player_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    mut shift: ResMut<ShiftState>,
    mut player_query: Query<(&GridPos, &Facing, &mut Carrying, &Transform), With<Player>>,
    mut station_query: Query<(Entity, &mut Station, &GridPos)>,
    item_on_ground_query: Query<(Entity, &Item, &GridPos)>,
    solid_query: Query<&GridPos, (With<Solid>, Without<Player>)>,
    mut commands: Commands,
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

    for (_station_entity, mut station, station_pos) in station_query.iter_mut() {
        if *station_pos != front_pos {
            continue;
        }

        match carrying.entity {
            Some(_carried_entity) => {
                if carrying.kind == Some(ItemKind::Case)
                    && station.kind == StationKind::Palletizer
                {
                    commands.entity(_carried_entity).despawn();
                    carrying.entity = None;
                    carrying.kind = None;
                    shift.cases_completed += 1;
                    return;
                }

                if station.kind != StationKind::Source
                    && carrying.kind == Some(station.accepted_kind)
                    && !station.busy
                    && !station.has_output
                {
                    commands.entity(_carried_entity).despawn();
                    carrying.entity = None;
                    carrying.kind = None;

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
                            transform: Transform::from_translation(Vec3::new(
                                player_transform.translation.x,
                                player_transform.translation.y,
                                0.1,
                            )),
                            ..default()
                        },
                        GameEntity,
                    )).id();
                    carrying.entity = Some(item_entity);
                    carrying.kind = Some(station.output_kind);
                    station.has_output = false;
                    return;
                }
            }
        }
        return;
    }

    if carrying.entity.is_none() {
        for (item_entity, item, ground_pos) in item_on_ground_query.iter() {
            if *ground_pos == front_pos {
                carrying.entity = Some(item_entity);
                carrying.kind = Some(item.kind);
                commands.entity(item_entity).remove::<GridPos>();
                return;
            }
        }
    }

    if let Some(carried_entity) = carrying.entity {
        let blocked = solid_query.iter().any(|gp| *gp == front_pos)
            || station_query.iter().any(|(_, _, gp)| *gp == front_pos)
            || item_on_ground_query.iter().any(|(_, _, gp)| *gp == front_pos);

        if !blocked {
            commands.entity(carried_entity).insert(front_pos);
            carrying.entity = None;
            carrying.kind = None;
        }
    }
}

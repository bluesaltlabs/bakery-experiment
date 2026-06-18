use bevy::prelude::*;
use crate::components::*;
use crate::level::{grid_to_world, TILE_SIZE};
use crate::resources::ConveyorTimerResource;

pub fn process_stations(
    time: Res<Time>,
    mut station_query: Query<&mut Station>,
) {
    for mut station in station_query.iter_mut() {
        if station.kind == StationKind::Source && !station.has_output {
            station.spawn_timer += time.delta_seconds();
            if station.spawn_timer >= station.spawn_interval {
                station.has_output = true;
                station.spawn_timer = 0.0;
            }
        }

        if station.busy {
            station.timer += time.delta_seconds();
            if station.timer >= station.process_duration {
                station.busy = false;
                station.timer = 0.0;
                station.has_output = true;
            }
        }
    }
}

pub fn process_conveyors(
    time: Res<Time>,
    mut conveyor_timer: ResMut<ConveyorTimerResource>,
    conveyor_query: Query<(&ConveyorBelt, &GridPos), Without<Item>>,
    solid_query: Query<&GridPos, (With<Solid>, Without<Item>, Without<Player>)>,
    station_query: Query<&GridPos, (With<Station>, Without<Item>, Without<Player>)>,
    mut item_params: ParamSet<(
        Query<&GridPos, (With<Item>, Without<Player>)>,
        Query<(Entity, &mut GridPos), (With<Item>, Without<Player>)>,
    )>,
    mut commands: Commands,
) {
    conveyor_timer.0.tick(time.delta());
    if !conveyor_timer.0.finished() {
        return;
    }

    let belts: Vec<(GridPos, crate::components::Direction)> = conveyor_query
        .iter()
        .map(|(belt, gp)| (*gp, belt.direction))
        .collect();

    let occupied: Vec<GridPos> = item_params.p0().iter().copied().collect();

    for (entity, mut item_pos) in item_params.p1().iter_mut() {
        if let Some((_, dir)) = belts.iter().find(|(bp, _)| *bp == *item_pos) {
            let delta = dir.delta();
            let next_pos = GridPos {
                x: item_pos.x + delta.0,
                y: item_pos.y + delta.1,
            };

            let blocked = solid_query.iter().any(|gp| *gp == next_pos)
                || station_query.iter().any(|gp| *gp == next_pos)
                || occupied.iter().any(|gp| *gp == next_pos);

            if !blocked {
                item_pos.x = next_pos.x;
                item_pos.y = next_pos.y;
                commands.entity(entity).remove::<FloorTimer>();
            }
        }
    }
}

pub fn sync_ground_items(
    mut item_query: Query<(&GridPos, &mut Transform), (With<Item>, Changed<GridPos>)>,
    conveyor_query: Query<&GridPos, (With<ConveyorBelt>, Without<Item>)>,
) {
    for (pos, mut transform) in item_query.iter_mut() {
        let mut p = grid_to_world(*pos);
        if conveyor_query.iter().any(|gp| *gp == *pos) {
            p.z = 0.06;
        }
        transform.translation = p;
    }
}

pub fn update_station_labels(
    station_query: Query<&Station>,
    mut label_query: Query<(&StationLabel, &mut Text)>,
) {
    for (label, mut text) in label_query.iter_mut() {
        if let Ok(station) = station_query.get(label.station_entity) {
            if station.kind == StationKind::Packer && !station.busy {
                text.sections[0].value = format!(
                    "Packer {}/{}",
                    station.packer_count,
                    station.inputs_needed,
                );
            } else {
                text.sections[0].value = station.kind.label();
            }
        }
    }
}
pub fn tick_floor_timers(
    time: Res<Time>,
    mut commands: Commands,
    mut item_query: Query<(Entity, &mut FloorTimer)>,
) {
    for (entity, mut timer) in item_query.iter_mut() {
        timer.0 -= time.delta_seconds();
        if timer.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
pub fn update_station_visuals(
    mut query: Query<(&Station, &mut Sprite)>,
) {
    for (station, mut sprite) in query.iter_mut() {
        sprite.color = if station.has_output {
            station.kind.color_ready()
        } else if station.busy {
            let t = (station.timer / station.process_duration).clamp(0.0, 1.0);
            station.kind.color_idle().mix(&station.kind.color_busy(), t)
        } else {
            station.kind.color_idle()
        };
    }
}

#[allow(dead_code)]
fn station_output_ready(
    station_query: &Query<(Entity, &Station, &GridPos)>,
    pos: GridPos,
    kind: StationKind,
) -> bool {
    station_query
        .iter()
        .any(|(_, s, sp)| *sp == pos && s.kind == kind && s.has_output && !s.busy)
}

pub fn animate_conveyors(
    time: Res<Time>,
    mut arrow_query: Query<(&mut Transform, &ConveyorArrow)>,
) {
    let half = TILE_SIZE * 0.4;
    let bar_half = TILE_SIZE * 0.08;
    let range = half - bar_half;
    let steps = 3.0;
    let speed = 1.5;
    let t = (time.elapsed_seconds() * speed) % 1.0;
    let step = (t * steps).floor() / steps;
    let progress = 1.0 - step;

    for (mut transform, arrow) in arrow_query.iter_mut() {
        let delta = 2.0 * range * progress;
        match arrow.direction {
            crate::components::Direction::Down => {
                transform.translation.x = arrow.base.x;
                transform.translation.y = arrow.base.y + delta;
            }
            crate::components::Direction::Up => {
                transform.translation.x = arrow.base.x;
                transform.translation.y = arrow.base.y - delta;
            }
            crate::components::Direction::Left => {
                transform.translation.x = arrow.base.x + delta;
                transform.translation.y = arrow.base.y;
            }
            crate::components::Direction::Right => {
                transform.translation.x = arrow.base.x - delta;
                transform.translation.y = arrow.base.y;
            }
        }
    }
}

use bevy::prelude::*;
use crate::components::*;
use crate::level::grid_to_world;

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

pub fn sync_ground_items(
    mut item_query: Query<(&GridPos, &mut Transform), (With<Item>, Changed<GridPos>)>,
) {
    for (pos, mut transform) in item_query.iter_mut() {
        transform.translation = grid_to_world(*pos);
    }
}

pub fn update_station_visuals(
    mut query: Query<(&Station, &mut Sprite)>,
) {
    for (station, mut sprite) in query.iter_mut() {
        sprite.color = if station.has_output {
            station.kind.color_ready()
        } else if station.busy {
            station.kind.color_busy()
        } else {
            station.kind.color_idle()
        };
    }
}

#![cfg_attr(target_arch = "wasm32", no_main)]

mod components;
mod interaction;
mod level;
mod movement;
mod player;
mod resources;
mod stations;
mod ui;

use std::time::Duration;
use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use bevy::audio::AudioPlugin;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bakery Puzzle-Sim".into(),
                    visible: true,
                    ..default()
                }),
                ..default()
            })
            .build()
            .disable::<AudioPlugin>(),
    );

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Bakery Puzzle-Sim".into(),
            visible: true,
            ..default()
        }),
        ..default()
    }));

    app.insert_resource(resources::ShiftState::new())
        .insert_resource(resources::MovementCooldown({
            let mut t = Timer::from_seconds(0.15, TimerMode::Once);
            t.tick(Duration::from_secs_f32(1.0));
            t
        }))
        .insert_resource(resources::GridVisible(true))
        .add_systems(Startup, (setup_camera, setup_level_sys, spawn_player_sys, setup_ui_sys))
        .add_systems(
            Update,
            (
                movement::player_movement,
                interaction::player_interaction,
                interaction::update_carried_items,
                stations::process_stations,
                stations::sync_ground_items,
                stations::update_station_visuals,
                stations::update_station_labels,
                player::update_direction_indicator,
                level::toggle_grid,
                ui::update_game_state,
                ui::update_ui,
                ui::show_game_over,
                ui::handle_restart,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(
            level::MAP_WIDTH as f32 * level::TILE_SIZE / 2.0,
            level::MAP_HEIGHT as f32 * level::TILE_SIZE / 2.0,
            1000.0,
        )),
        ..default()
    });
}

fn setup_level_sys(mut commands: Commands) {
    level::setup_level(&mut commands);
}

fn spawn_player_sys(mut commands: Commands) {
    player::spawn_player(&mut commands);
}

fn setup_ui_sys(mut commands: Commands) {
    ui::setup_ui(&mut commands);
}

#![cfg_attr(target_arch = "wasm32", no_main)]

mod audio;
mod components;
mod interaction;
mod level;
mod movement;
mod npc;
mod player;
mod resources;
mod stations;
mod ui;

use std::time::Duration;
use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use components::{Direction, GridPos};
use resources::ConveyorTimerResource;

fn make_window() -> Window {
    Window {
        title: "Bakery Puzzle-Sim".into(),
        visible: true,
        ..default()
    }
}

fn build_app() -> App {
    let mut app = App::new();
    let window = WindowPlugin {
        primary_window: Some(make_window()),
        ..default()
    };

    #[cfg(target_arch = "wasm32")] {
        use bevy::audio::AudioPlugin;
        app.add_plugins(DefaultPlugins.set(window).build().disable::<AudioPlugin>());
    }
    #[cfg(not(target_arch = "wasm32"))] {
        app.add_plugins(DefaultPlugins.set(window));
    }

    app
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    let mut app = build_app();

    audio::setup_audio_system(&mut app);

    app.insert_resource(resources::ShiftState::new())
        .insert_resource(resources::MovementCooldown({
            let mut t = Timer::from_seconds(0.15, TimerMode::Once);
            t.tick(Duration::from_secs_f32(1.0));
            t
        }))
        .insert_resource(resources::GridVisible(true))
        .insert_resource(ConveyorTimerResource(Timer::from_seconds(
            0.5,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, (setup_camera, setup_level_sys, spawn_player_sys, spawn_npc_sys, setup_ui_sys))
        .add_systems(
            Update,
            (
                movement::player_movement,
                stations::process_conveyors,
                stations::animate_conveyors,
                interaction::player_interaction,
            ),
        )
        .add_systems(
            Update,
            (
                npc::conveyor_loader::conveyor_loader_ai,
                npc::oven_hauler::oven_hauler_ai,
                npc::packer_hauler::packer_hauler_ai,
                npc::update_npc_direction_indicator,
                interaction::update_carried_items,
                stations::process_stations,
                stations::sync_ground_items,
                stations::update_station_visuals,
                stations::update_station_labels,
                stations::tick_floor_timers,
                player::update_direction_indicator,
                level::toggle_grid,
                ui::update_game_state,
                ui::update_ui,
                ui::show_game_over,
                ui::handle_restart,
                ui::keyboard_volume_control,
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

fn spawn_npc_sys(mut commands: Commands) {
    use bevy::color::Color;
    crate::npc::spawn_conveyor_loader(
        &mut commands,
        GridPos { x: 4, y: 4 },
        Color::srgb(1.0, 0.5, 0.0),
        Color::srgb(1.0, 0.7, 0.3),
        Direction::Left,
        1.0,
        0.5,
    );
    crate::npc::spawn_oven_hauler(
        &mut commands,
        GridPos { x: 5, y: 2 },
        Color::srgb(0.2, 0.8, 0.5),
        Color::srgb(0.4, 1.0, 0.6),
        Direction::Left,
        0.5,
        0.25,
    );
    crate::npc::spawn_packer_hauler(
        &mut commands,
        GridPos { x: 7, y: 5 },
        Color::srgb(0.3, 0.5, 0.9),
        Color::srgb(0.5, 0.7, 1.0),
        Direction::Right,
        0.5,
        0.25,
    );
}

fn setup_ui_sys(mut commands: Commands) {
    ui::setup_ui(&mut commands);
}

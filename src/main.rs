#![cfg_attr(target_arch = "wasm32", no_main)]

mod audio;
mod components;
mod editor;
mod interaction;
mod level;
mod mobile;
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
use components::{GridPos, Player};
use mobile::{MobileInput, MobileOverlayVisible};
use resources::{ConveyorTimerResource, EditorMode, LevelData, SelectedTile};

fn make_window() -> Window {
    Window {
        title: "Bakery Puzzle-Sim".into(),
        visible: true,
        #[cfg(target_arch = "wasm32")]
        fit_canvas_to_parent: true,
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
        .insert_resource(MobileInput::default())
        .insert_resource(MobileOverlayVisible(true))
        .insert_resource(LevelData::new())
        .insert_resource(EditorMode(false))
        .insert_resource(SelectedTile(1))
        .insert_resource(editor::RebuildRequested(false))
        .add_systems(Startup, (setup_camera, setup_level_sys, spawn_player_sys, spawn_npc_sys, setup_ui_sys, mobile::setup_mobile_overlay, editor::setup_editor_ui, editor::setup_editor_cursor))
        .add_systems(
            Update,
            (
                mobile::handle_tap_to_move,
                mobile::handle_overlay_buttons,
                movement::player_movement,
                adjust_camera_scale,
                camera_follow.after(movement::player_movement),
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
                mobile::handle_overlay_toggle,
                level::toggle_grid,
                ui::update_game_state,
                ui::update_ui,
                ui::show_game_over,
                ui::handle_restart,
                ui::keyboard_volume_control,
            ),
        )
        .add_systems(
            Update,
            (
                editor::toggle_editor_mode,
                editor::editor_camera_pan,
                editor::update_editor_cursor,
                editor::editor_place_tile,
                editor::handle_palette_buttons,
                editor::update_palette_highlight,
                editor::update_palette_visibility,
                editor::editor_palette_keyboard,
                editor::rebuild_level,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    let player_world = level::grid_to_world(GridPos { x: 1, y: 1 });
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(
            player_world.x,
            player_world.y,
            1000.0,
        )),
        ..default()
    });
}

fn adjust_camera_scale(
    windows: Query<&Window>,
    mut camera_query: Query<&mut OrthographicProjection, With<Camera>>,
) {
    let Ok(window) = windows.get_single() else { return };
    let Ok(mut projection) = camera_query.get_single_mut() else { return };

    let map_w = level::MAP_WIDTH as f32 * level::TILE_SIZE;
    let map_h = level::MAP_HEIGHT as f32 * level::TILE_SIZE;
    let min_scale = 0.75;

    let ideal_scale = (window.width() / map_w).min(window.height() / map_h) * min_scale;
    projection.scale = ideal_scale.clamp(min_scale, 1.0);
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &OrthographicProjection), (With<Camera>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(player) = player_query.get_single() else { return };
    let Ok((mut camera, projection)) = camera_query.get_single_mut() else { return };

    let dead_zone_half = 3.0 * level::TILE_SIZE / 2.0;

    let offset = player.translation.truncate() - camera.translation.truncate();

    let excess = Vec2::new(
        (offset.x.abs() - dead_zone_half).max(0.0) * offset.x.signum(),
        (offset.y.abs() - dead_zone_half).max(0.0) * offset.y.signum(),
    );

    let mut target = camera.translation.truncate() + excess;

    let half_w = projection.area.half_size().x;
    let half_h = projection.area.half_size().y;
    let map_w = level::MAP_WIDTH as f32 * level::TILE_SIZE;
    let map_h = level::MAP_HEIGHT as f32 * level::TILE_SIZE;
    if half_w * 2.0 >= map_w {
        target.x = map_w / 2.0;
    } else {
        target.x = target.x.clamp(half_w, map_w - half_w);
    }
    if half_h * 2.0 >= map_h {
        target.y = map_h / 2.0;
    } else {
        target.y = target.y.clamp(half_h, map_h - half_h);
    }

    let t = (8.0 * time.delta_seconds()).clamp(0.0, 1.0);
    camera.translation.x += (target.x - camera.translation.x) * t;
    camera.translation.y += (target.y - camera.translation.y) * t;
}

fn setup_level_sys(mut commands: Commands, level_data: Res<LevelData>) {
    level::setup_level(&mut commands, &level_data);
}

fn spawn_player_sys(mut commands: Commands) {
    player::spawn_player(&mut commands);
}

fn spawn_npc_sys(mut commands: Commands, level_data: Res<LevelData>) {
    for npc_data in &level_data.npcs {
        crate::npc::spawn_npc_from_data(&mut commands, npc_data);
    }
}

fn setup_ui_sys(mut commands: Commands) {
    ui::setup_ui(&mut commands);
}

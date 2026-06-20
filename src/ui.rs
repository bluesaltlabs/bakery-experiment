use bevy::prelude::*;
use crate::audio::{AudioEvent, UserVolume};
use crate::components::*;
use crate::level::*;
use crate::mobile::MobileInput;
use crate::resources::{EditorMode, GridVisible, LevelData, ShiftState};

pub fn update_game_state(
    editor: Res<EditorMode>,
    time: Res<Time>,
    mut shift: ResMut<ShiftState>,
    mut audio_queue: ResMut<crate::audio::AudioEventQueue>,
) {
    if editor.0 || shift.game_over {
        return;
    }

    shift.time_remaining -= time.delta_seconds();

    if shift.time_remaining <= 30.0 && shift.time_remaining > 0.0
        && shift.time_remaining.floor() != (shift.time_remaining + time.delta_seconds()).floor()
    {
        audio_queue.0.push(AudioEvent::TimerWarning);
    }

    if shift.time_remaining <= 0.0 {
        shift.time_remaining = 0.0;
        shift.game_over = true;
        shift.victory = shift.cases_completed >= shift.target_cases;
        if shift.victory {
            audio_queue.0.push(AudioEvent::Win);
        } else {
            audio_queue.0.push(AudioEvent::Lose);
        }
    } else if shift.cases_completed >= shift.target_cases {
        shift.game_over = true;
        shift.victory = true;
        audio_queue.0.push(AudioEvent::Win);
    }
}

pub fn handle_restart(
    keys: Res<ButtonInput<KeyCode>>,
    mut mobile_input: ResMut<MobileInput>,
    mut shift: ResMut<ShiftState>,
    mut grid_visible: ResMut<GridVisible>,
    game_entities: Query<Entity, With<GameEntity>>,
    level_data: Res<LevelData>,
    mut commands: Commands,
) {
    let restart = keys.just_pressed(KeyCode::KeyR) || mobile_input.restart;
    mobile_input.restart = false;
    if !restart {
        return;
    }

    for entity in game_entities.iter() {
        commands.entity(entity).despawn();
    }

    *shift = ShiftState::new();
    grid_visible.0 = true;
    setup_level(&mut commands, &level_data);
    crate::player::spawn_player(&mut commands);
    for npc_data in &level_data.npcs {
        crate::npc::spawn_npc_from_data(&mut commands, npc_data);
    }
    setup_ui(&mut commands);
}

pub fn setup_ui(commands: &mut Commands) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Time: 300",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "\nCases: 0/10",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "\nCarrying: None",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.8, 0.8, 0.8),
                    ..default()
                },
            ),
            TextSection::new(
                "\nVolume: 25%",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.6, 0.6, 0.9),
                    ..default()
                },
            ),
            TextSection::new(
                "\n\nStations:",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.7, 0.7, 0.7),
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Vh(1.0),
            left: Val::Vw(1.0),
            ..default()
        }),
        GameEntity,
        HudText,
    ));

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 36.0,
                color: Color::srgb(1.0, 1.0, 0.0),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Vh(45.0),
            left: Val::Vw(35.0),
            ..default()
        }),
        GameEntity,
        GameOverText,
    ));
}

pub fn keyboard_volume_control(
    keys: Res<ButtonInput<KeyCode>>,
    mut volume: ResMut<UserVolume>,
) {
    if keys.just_pressed(KeyCode::Comma) {
        volume.0 = (volume.0 - 0.05).max(0.0);
    }
    if keys.just_pressed(KeyCode::Period) {
        volume.0 = (volume.0 + 0.05).min(1.0);
    }
}

pub fn update_ui(
    shift: Res<ShiftState>,
    player_query: Query<&Carrying, With<Player>>,
    mut text_query: Query<&mut Text, With<HudText>>,
    station_query: Query<&Station>,
    volume: Res<UserVolume>,
) {
    if shift.game_over {
        return;
    }

    let carrying_name = match player_query.iter().next() {
        Some(carrying) => match carrying.0 {
            Some((_, kind)) => kind.label().to_string(),
            None => "None".to_string(),
        },
        None => "None".to_string(),
    };

    if let Some(mut text) = text_query.iter_mut().next() {
        text.sections[0].value = format!("Time: {:.0}", shift.time_remaining);
        text.sections[1].value = format!("\nCases: {}/{}", shift.cases_completed, shift.target_cases);
        text.sections[2].value = format!("\nCarrying: {}", carrying_name);
        text.sections[3].value = format!("\nVolume: {:.0}%", volume.0 * 100.0);

        let mut stations: Vec<&Station> = station_query.iter().collect();
        stations.sort_by(|a, b| {
            fn order(k: &StationKind) -> u8 {
                match k {
                    StationKind::Source => 0,
                    StationKind::Former => 1,
                    StationKind::Oven => 2,
                    StationKind::Packer => 3,
                    StationKind::Palletizer => 4,
                    StationKind::Table => 5,
                }
            }
            order(&a.kind).cmp(&order(&b.kind))
        });

        let mut debug = String::from("\n\nStations:");
        for station in stations {
            let pct = if station.busy {
                (station.timer / station.process_duration * 100.0) as u32
            } else if station.kind == StationKind::Source {
                (station.spawn_timer / station.spawn_interval * 100.0) as u32
            } else {
                0
            };
            let status = if station.has_output {
                station.output_kind.label()
            } else if station.busy {
                "processing"
            } else if station.packer_count > 0 {
                "partial"
            } else {
                "empty"
            };
            debug.push_str(&format!(
                "\n  {}: {}",
                station.kind.label(),
                status,
            ));
            if station.busy {
                debug.push_str(&format!(" ({}%)", pct));
            } else if station.kind == StationKind::Source && pct > 0 {
                debug.push_str(&format!(" ({}%)", pct));
            }
            if station.kind == StationKind::Packer && station.packer_count > 0 {
                debug.push_str(&format!(" {}/{}", station.packer_count, station.inputs_needed));
            }
        }
        text.sections[4].value = debug;
    }
}

pub fn show_game_over(
    shift: Res<ShiftState>,
    mut text_query: Query<&mut Text, With<GameOverText>>,
) {
    if !shift.game_over {
        return;
    }

    if let Some(mut text) = text_query.iter_mut().next() {
        if shift.victory {
            text.sections[0].value = "You Win!\nPress R to Restart".to_string();
            text.sections[0].style.color = Color::srgb(0.0, 1.0, 0.0);
        } else {
            text.sections[0].value = "Shift Over\nPress R to Restart".to_string();
            text.sections[0].style.color = Color::srgb(1.0, 0.3, 0.3);
        }
    }
}

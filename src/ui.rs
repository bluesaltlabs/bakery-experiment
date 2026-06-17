use bevy::prelude::*;
use crate::components::*;
use crate::resources::{GridVisible, ShiftState};
use crate::level::*;

pub fn update_game_state(
    time: Res<Time>,
    mut shift: ResMut<ShiftState>,
) {
    if shift.game_over {
        return;
    }

    shift.time_remaining -= time.delta_seconds();
    if shift.time_remaining <= 0.0 {
        shift.time_remaining = 0.0;
        shift.game_over = true;
        shift.victory = shift.cases_completed >= shift.target_cases;
    }
    if shift.cases_completed >= shift.target_cases {
        shift.game_over = true;
        shift.victory = true;
    }
}

pub fn handle_restart(
    keys: Res<ButtonInput<KeyCode>>,
    mut shift: ResMut<ShiftState>,
    mut grid_visible: ResMut<GridVisible>,
    game_entities: Query<Entity, With<GameEntity>>,
    mut commands: Commands,
) {
    if !keys.just_pressed(KeyCode::KeyR) {
        return;
    }

    for entity in game_entities.iter() {
        commands.entity(entity).despawn();
    }

    *shift = ShiftState::new();
    grid_visible.0 = true;
    setup_level(&mut commands);
    crate::player::spawn_player(&mut commands);
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
                "\nCases: 0/20",
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
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
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
            top: Val::Px(300.0),
            left: Val::Px(250.0),
            ..default()
        }),
        GameEntity,
        GameOverText,
    ));
}

pub fn update_ui(
    shift: Res<ShiftState>,
    player_query: Query<&Carrying, With<Player>>,
    mut text_query: Query<&mut Text, With<HudText>>,
) {
    if shift.game_over {
        return;
    }

    let carrying_name = match player_query.iter().next() {
        Some(carrying) => match carrying.kind {
            Some(kind) => kind.label().to_string(),
            None => "None".to_string(),
        },
        None => "None".to_string(),
    };

    if let Some(mut text) = text_query.iter_mut().next() {
        text.sections[0].value = format!("Time: {:.0}", shift.time_remaining);
        text.sections[1].value = format!("\nCases: {}/{}", shift.cases_completed, shift.target_cases);
        text.sections[2].value = format!("\nCarrying: {}", carrying_name);
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

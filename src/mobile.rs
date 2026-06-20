use bevy::prelude::*;
use crate::components::{ConveyorBelt, Direction, Facing, GridPos, Item, Player, Station};
use crate::level::{TILE_SIZE, MAP_WIDTH, MAP_HEIGHT};
use crate::resources::{EditorMode, ShiftState};

pub(crate) const OVERLAY_WIDTH: f32 = 76.0;

#[derive(Resource, Default)]
pub struct MobileInput {
    pub direction: Option<Direction>,
    pub interact: bool,
    pub restart: bool,
    pub toggle_grid: bool,
    pub toggle_editor: bool,
}

#[derive(Resource)]
pub struct MobileOverlayVisible(pub bool);

#[derive(Component)]
pub struct MobileOverlayRoot;

#[derive(Component)]
pub struct MobileInteractButton;

#[derive(Component)]
pub struct MobileRestartButton;

#[derive(Component)]
pub struct MobileGridButton;

#[derive(Component)]
pub struct MobileEditorButton;

#[derive(Component)]
pub struct MobileHideButton;

#[derive(Component)]
pub struct MobileShowButton;

#[derive(Component)]
pub struct MobileUpButton;

#[derive(Component)]
pub struct MobileDownButton;

#[derive(Component)]
pub struct MobileLeftButton;

#[derive(Component)]
pub struct MobileRightButton;

pub fn handle_tap_to_move(
    editor: Res<EditorMode>,
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    player_query: Query<(&GridPos, &Facing), With<Player>>,
    station_query: Query<&GridPos, (With<Station>, Without<Player>)>,
    conveyor_query: Query<&GridPos, (With<ConveyorBelt>, Without<Player>)>,
    item_query: Query<&GridPos, (With<Item>, Without<Player>)>,
    shift: Res<ShiftState>,
    mut mobile_input: ResMut<MobileInput>,
) {
    if editor.0 || shift.game_over {
        return;
    }

    let tap_pos = touches.iter_just_pressed().next().map(|t| t.position()).or_else(|| {
        if mouse.just_pressed(MouseButton::Left) {
            windows.single().cursor_position()
        } else {
            None
        }
    });
    let Some(tap_pos) = tap_pos else { return };

    let window = windows.single();
    if tap_pos.x > window.width() - OVERLAY_WIDTH {
        return;
    }

    let Ok((camera, camera_transform)) = cameras.get_single() else { return };
    let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, tap_pos) else { return };

    let grid_x = (world_pos.x / TILE_SIZE).floor() as i32;
    let grid_y = (world_pos.y / TILE_SIZE).floor() as i32;
    let tap_grid = GridPos {
        x: grid_x.clamp(0, MAP_WIDTH as i32 - 1),
        y: grid_y.clamp(0, MAP_HEIGHT as i32 - 1),
    };

    let Ok((player_pos, facing)) = player_query.get_single() else { return };

    let dx = tap_grid.x - player_pos.x;
    let dy = tap_grid.y - player_pos.y;

    if dx == 0 && dy == 0 {
        return;
    }

    let front_pos = GridPos {
        x: player_pos.x + facing.0.delta().0,
        y: player_pos.y + facing.0.delta().1,
    };

    if tap_grid == front_pos {
        let is_interactable = station_query.iter().any(|gp| *gp == front_pos)
            || conveyor_query.iter().any(|gp| *gp == front_pos)
            || item_query.iter().any(|gp| *gp == front_pos);
        if is_interactable {
            mobile_input.interact = true;
            return;
        }
    }

    mobile_input.direction = Some(if dx.abs() >= dy.abs() {
        if dx > 0 { Direction::Right } else { Direction::Left }
    } else {
        if dy > 0 { Direction::Up } else { Direction::Down }
    });
}

pub fn handle_overlay_buttons(
    interact_query: Query<&Interaction, (With<MobileInteractButton>, Changed<Interaction>)>,
    restart_query: Query<&Interaction, (With<MobileRestartButton>, Changed<Interaction>)>,
    grid_query: Query<&Interaction, (With<MobileGridButton>, Changed<Interaction>)>,
    editor_query: Query<&Interaction, (With<MobileEditorButton>, Changed<Interaction>)>,
    up_query: Query<&Interaction, (With<MobileUpButton>, Changed<Interaction>)>,
    down_query: Query<&Interaction, (With<MobileDownButton>, Changed<Interaction>)>,
    left_query: Query<&Interaction, (With<MobileLeftButton>, Changed<Interaction>)>,
    right_query: Query<&Interaction, (With<MobileRightButton>, Changed<Interaction>)>,
    mut mobile_input: ResMut<MobileInput>,
) {
    if interact_query.iter().any(|i| *i == Interaction::Pressed) {
        mobile_input.interact = true;
    }
    if restart_query.iter().any(|i| *i == Interaction::Pressed) {
        mobile_input.restart = true;
    }
    if grid_query.iter().any(|i| *i == Interaction::Pressed) {
        mobile_input.toggle_grid = true;
    }
    if editor_query.iter().any(|i| *i == Interaction::Pressed) {
        mobile_input.toggle_editor = true;
    }

    let up = up_query.iter().any(|i| *i == Interaction::Pressed);
    let down = down_query.iter().any(|i| *i == Interaction::Pressed);
    let left = left_query.iter().any(|i| *i == Interaction::Pressed);
    let right = right_query.iter().any(|i| *i == Interaction::Pressed);

    if up || down || left || right {
        mobile_input.direction = if up { Some(Direction::Up) }
            else if down { Some(Direction::Down) }
            else if left { Some(Direction::Left) }
            else { Some(Direction::Right) };
    }
}

pub fn handle_overlay_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    hide_query: Query<&Interaction, (With<MobileHideButton>, Changed<Interaction>)>,
    show_query: Query<&Interaction, (With<MobileShowButton>, Changed<Interaction>)>,
    mut overlay_visible: ResMut<MobileOverlayVisible>,
    mut overlay_root_query: Query<&mut Visibility, (With<MobileOverlayRoot>, Without<MobileShowButton>)>,
    mut show_button_query: Query<&mut Visibility, (With<MobileShowButton>, Without<MobileOverlayRoot>)>,
) {
    let hide = hide_query.iter().any(|i| *i == Interaction::Pressed);
    let show = show_query.iter().any(|i| *i == Interaction::Pressed);
    let toggle_key = keys.just_pressed(KeyCode::KeyH);

    if hide || show || toggle_key {
        if hide || (toggle_key && overlay_visible.0) {
            overlay_visible.0 = false;
        } else {
            overlay_visible.0 = true;
        }

        if let Ok(mut vis) = overlay_root_query.get_single_mut() {
            *vis = if overlay_visible.0 { Visibility::Visible } else { Visibility::Hidden };
        }
        if let Ok(mut vis) = show_button_query.get_single_mut() {
            *vis = if overlay_visible.0 { Visibility::Hidden } else { Visibility::Visible };
        }
    }
}

fn arrow_style() -> Style {
    Style {
        width: Val::Px(60.0),
        height: Val::Px(36.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn half_arrow_style() -> Style {
    Style {
        width: Val::Px(30.0),
        height: Val::Px(36.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn button_style() -> Style {
    Style {
        width: Val::Px(60.0),
        height: Val::Px(56.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn text_style() -> TextStyle {
    TextStyle {
        font_size: 14.0,
        color: Color::WHITE,
        ..default()
    }
}

fn bg() -> BackgroundColor {
    BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.85))
}

pub fn setup_mobile_overlay(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(8.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(70.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.45)),
            visibility: Visibility::Visible,
            ..default()
        },
        MobileOverlayRoot,
    ))
    .with_children(|parent| {
        parent.spawn((
            ButtonBundle {
                style: arrow_style(),
                background_color: bg(),
                ..default()
            },
            MobileUpButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "^",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });

        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(60.0),
                height: Val::Px(36.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            background_color: BackgroundColor(Color::NONE),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                ButtonBundle {
                    style: half_arrow_style(),
                    background_color: bg(),
                    ..default()
                },
                MobileLeftButton,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "<",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });

            parent.spawn((
                ButtonBundle {
                    style: half_arrow_style(),
                    background_color: bg(),
                    ..default()
                },
                MobileRightButton,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    ">",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
        });

        parent.spawn((
            ButtonBundle {
                style: arrow_style(),
                background_color: bg(),
                ..default()
            },
            MobileDownButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "v",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });

        parent.spawn((
            ButtonBundle {
                style: button_style(),
                background_color: bg(),
                ..default()
            },
            MobileInteractButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "E\nInteract",
                text_style(),
            ));
        });

        parent.spawn((
            ButtonBundle {
                style: button_style(),
                background_color: bg(),
                ..default()
            },
            MobileRestartButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "R\nRestart",
                text_style(),
            ));
        });

        parent.spawn((
            ButtonBundle {
                style: button_style(),
                background_color: bg(),
                ..default()
            },
            MobileGridButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "G\nGrid",
                text_style(),
            ));
        });

        parent.spawn((
            ButtonBundle {
                style: button_style(),
                background_color: bg(),
                ..default()
            },
            MobileEditorButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "F2\nEdit",
                text_style(),
            ));
        });

        parent.spawn((
            ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(24.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: bg(),
                ..default()
            },
            MobileHideButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "<",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
    });

    commands.spawn((
        ButtonBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(24.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: bg(),
            visibility: Visibility::Hidden,
            ..default()
        },
        MobileShowButton,
    ))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            ">",
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

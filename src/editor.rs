use bevy::prelude::*;
use crate::components::{GameEntity, GridPos, Player};
use crate::level::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};
use crate::resources::{EditorMode, LevelData, SelectedTile};

const PALETTE_WIDTH: f32 = 80.0;

#[derive(Resource)]
pub struct RebuildRequested(pub bool);

#[derive(Component)]
pub struct EditorPaletteRoot;

#[derive(Component)]
pub struct TileButton(pub u8, pub Color);

#[derive(Component)]
pub struct EditorCursor;

const TILE_INFO: &[(u8, &str, (f32, f32, f32))] = &[
    (0, "Erase", (0.3, 0.3, 0.3)),
    (1, "Wall", (0.3, 0.3, 0.35)),
    (2, "Src", (0.2, 0.8, 0.2)),
    (3, "Form", (0.8, 0.5, 0.2)),
    (4, "Oven", (0.9, 0.3, 0.1)),
    (5, "Pack", (0.3, 0.3, 0.8)),
    (6, "Pall", (0.8, 0.2, 0.8)),
    (7, "Tbl", (0.6, 0.4, 0.2)),
    (8, "Conv", (0.25, 0.35, 0.45)),
];

pub fn setup_editor_ui(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(PALETTE_WIDTH),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                row_gap: Val::Px(4.0),
                overflow: Overflow::clip(),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.92)),
            visibility: Visibility::Hidden,
            z_index: ZIndex::Global(100),
            ..default()
        },
        EditorPaletteRoot,
    ))
    .with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "Tiles",
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
        ));
        for &(tile, label, (r, g, b)) in TILE_INFO {
            let color = Color::srgb(r, g, b);
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(PALETTE_WIDTH - 8.0),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(color),
                    ..default()
                },
                TileButton(tile, color),
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    label,
                    TextStyle {
                        font_size: 10.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
        }
    });
}

pub fn setup_editor_cursor(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.25),
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
            visibility: Visibility::Hidden,
            ..default()
        },
        EditorCursor,
    ));
}

pub fn toggle_editor_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<EditorMode>,
    mut rebuild: ResMut<RebuildRequested>,
) {
    if keys.just_pressed(KeyCode::F2) || keys.just_pressed(KeyCode::Backquote) {
        editor.0 = !editor.0;
        if !editor.0 {
            rebuild.0 = true;
        }
    }
}

pub fn editor_camera_pan(
    editor: Res<EditorMode>,
    time: Res<Time>,
    windows: Query<&Window>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    if !editor.0 {
        return;
    }
    let window = windows.single();
    let Some(cursor) = window.cursor_position() else { return };
    let (w, h) = (window.width(), window.height());
    let edge = 30.0;
    let speed = 500.0;

    let mut dx = 0.0f32;
    let mut dy = 0.0f32;

    if cursor.x < edge {
        dx = -1.0;
    }
    if cursor.x > w - edge {
        dx = 1.0;
    }
    if cursor.y < edge {
        dy = 1.0;
    }
    if cursor.y > h - edge {
        dy = -1.0;
    }

    if dx != 0.0 || dy != 0.0 {
        let dt = time.delta_seconds();
        for mut transform in camera_query.iter_mut() {
            transform.translation.x += dx * speed * dt;
            transform.translation.y += dy * speed * dt;
        }
    }
}

pub fn update_editor_cursor(
    editor: Res<EditorMode>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut cursor_query: Query<(&mut Transform, &mut Visibility), With<EditorCursor>>,
) {
    let Ok((mut transform, mut visibility)) = cursor_query.get_single_mut() else { return };

    if !editor.0 {
        *visibility = Visibility::Hidden;
        return;
    }

    let window = windows.single();
    let Some(cursor) = window.cursor_position() else {
        *visibility = Visibility::Hidden;
        return;
    };

    if cursor.x < PALETTE_WIDTH {
        *visibility = Visibility::Hidden;
        return;
    }

    let Ok((camera, camera_transform)) = cameras.get_single() else { return };
    let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        *visibility = Visibility::Hidden;
        return;
    };

    let grid_x = (world_pos.x / TILE_SIZE).floor() * TILE_SIZE + TILE_SIZE / 2.0;
    let grid_y = (world_pos.y / TILE_SIZE).floor() * TILE_SIZE + TILE_SIZE / 2.0;

    transform.translation = Vec3::new(grid_x, grid_y, 100.0);
    *visibility = Visibility::Visible;
}

fn screen_to_grid(
    cursor: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<(i32, i32)> {
    let world_pos = camera.viewport_to_world_2d(camera_transform, cursor)?;
    let gx = (world_pos.x / TILE_SIZE).floor() as i32;
    let gy = (world_pos.y / TILE_SIZE).floor() as i32;
    if gx >= 0 && gx < MAP_WIDTH as i32 && gy >= 0 && gy < MAP_HEIGHT as i32 {
        Some((gx, gy))
    } else {
        None
    }
}

pub fn editor_place_tile(
    editor: Res<EditorMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    selected: Res<SelectedTile>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut level_data: ResMut<LevelData>,
    mut rebuild: ResMut<RebuildRequested>,
) {
    if !editor.0 {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let Some(cursor) = window.cursor_position() else { return };

    if cursor.x < PALETTE_WIDTH {
        return;
    }

    let Ok((camera, camera_transform)) = cameras.get_single() else { return };
    let Some((gx, gy)) = screen_to_grid(cursor, camera, camera_transform) else { return };

    let row = (MAP_HEIGHT as i32 - 1 - gy) as usize;
    let col = gx as usize;

    level_data.tiles[row][col] = selected.0;
    rebuild.0 = true;
}

pub fn handle_palette_buttons(
    mut selected: ResMut<SelectedTile>,
    buttons: Query<(&Interaction, &TileButton), Changed<Interaction>>,
) {
    for (interaction, button) in buttons.iter() {
        if *interaction == Interaction::Pressed {
            selected.0 = button.0;
        }
    }
}

pub fn update_palette_highlight(
    selected: Res<SelectedTile>,
    mut buttons: Query<(&TileButton, &mut BackgroundColor)>,
) {
    for (button, mut bg) in buttons.iter_mut() {
        if button.0 == selected.0 {
            bg.0 = Color::WHITE;
        } else {
            bg.0 = button.1;
        }
    }
}

pub fn update_palette_visibility(
    editor: Res<EditorMode>,
    mut query: Query<&mut Visibility, With<EditorPaletteRoot>>,
) {
    for mut vis in query.iter_mut() {
        *vis = if editor.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn editor_palette_keyboard(
    editor: Res<EditorMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedTile>,
) {
    if !editor.0 {
        return;
    }
    if keys.just_pressed(KeyCode::Digit0) {
        selected.0 = 0;
    } else if keys.just_pressed(KeyCode::Digit1) {
        selected.0 = 1;
    } else if keys.just_pressed(KeyCode::Digit2) {
        selected.0 = 2;
    } else if keys.just_pressed(KeyCode::Digit3) {
        selected.0 = 3;
    } else if keys.just_pressed(KeyCode::Digit4) {
        selected.0 = 4;
    } else if keys.just_pressed(KeyCode::Digit5) {
        selected.0 = 5;
    } else if keys.just_pressed(KeyCode::Digit6) {
        selected.0 = 6;
    } else if keys.just_pressed(KeyCode::Digit7) {
        selected.0 = 7;
    } else if keys.just_pressed(KeyCode::Digit8) {
        selected.0 = 8;
    }
}

pub fn rebuild_level(
    mut rebuild: ResMut<RebuildRequested>,
    mut commands: Commands,
    level_data: Res<LevelData>,
    game_entities: Query<Entity, With<GameEntity>>,
    player_query: Query<&GridPos, With<Player>>,
) {
    if !rebuild.0 {
        return;
    }
    rebuild.0 = false;

    let player_pos = player_query
        .get_single()
        .copied()
        .unwrap_or(GridPos { x: 1, y: 1 });

    let entities: Vec<Entity> = game_entities.iter().collect();
    for entity in entities {
        commands.entity(entity).despawn();
    }

    crate::level::setup_level(&mut commands, &level_data);
    crate::player::spawn_player_at(&mut commands, player_pos);
    for npc_data in &level_data.npcs {
        crate::npc::spawn_npc_from_data(&mut commands, npc_data);
    }
    crate::ui::setup_ui(&mut commands);
}

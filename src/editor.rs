use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use crate::components::{Direction, GameEntity, GridPos, NpcKind, Player};
use crate::level::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE, CONVEYOR, Z_EDITOR_CURSOR};
use crate::mobile::{MobileInput, OVERLAY_WIDTH};
use crate::resources::{EditorMode, LevelData, SelectedNpc, SelectedTile, UndoEntry, UndoStack};

const PALETTE_WIDTH: f32 = 80.0;

#[derive(Resource)]
pub struct RebuildRequested(pub bool);

#[derive(Component)]
pub struct EditorPaletteRoot;

#[derive(Component)]
pub struct TileButton(pub u8, pub Color);

#[derive(Component)]
pub struct NpcPaletteButton(pub NpcKind, pub Color);

#[derive(Component)]
pub struct EditorSaveButton;

#[derive(Component)]
pub struct EditorLoadButton;

#[derive(Component)]
pub struct EditorResetButton;

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

const NPC_INFO: &[(NpcKind, &str, (f32, f32, f32))] = &[
    (NpcKind::ConveyorLoader, "CL", (1.0, 0.5, 0.0)),
    (NpcKind::OvenHauler, "OH", (0.2, 0.8, 0.5)),
    (NpcKind::PackerHauler, "PH", (0.3, 0.5, 0.9)),
];

fn spawn_palette_button(
    parent: &mut ChildBuilder,
    label: &str,
    color: Color,
    component: impl Component,
) {
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
        component,
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
        parent.spawn(TextBundle::from_section(
            "Tiles",
            TextStyle {
                font_size: 12.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        for &(tile, label, (r, g, b)) in TILE_INFO {
            spawn_palette_button(parent, label, Color::srgb(r, g, b), TileButton(tile, Color::srgb(r, g, b)));
        }

        parent.spawn(TextBundle::from_section(
            "NPCs",
            TextStyle {
                font_size: 12.0,
                color: Color::srgb(0.7, 0.7, 0.7),
                ..default()
            },
        ));
        for &(kind, label, (r, g, b)) in NPC_INFO {
            spawn_palette_button(parent, label, Color::srgb(r, g, b), NpcPaletteButton(kind, Color::srgb(r, g, b)));
        }

        spawn_palette_button(parent, "Save", Color::srgb(0.15, 0.5, 0.15), EditorSaveButton);
        spawn_palette_button(parent, "Load", Color::srgb(0.5, 0.3, 0.15), EditorLoadButton);
        spawn_palette_button(parent, "Reset", Color::srgb(0.6, 0.2, 0.2), EditorResetButton);
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
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, Z_EDITOR_CURSOR)),
            visibility: Visibility::Hidden,
            ..default()
        },
        EditorCursor,
    ));
}

pub fn toggle_editor_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut mobile_input: ResMut<MobileInput>,
    mut editor: ResMut<EditorMode>,
    mut rebuild: ResMut<RebuildRequested>,
    mut undo_stack: ResMut<UndoStack>,
) {
    if keys.just_pressed(KeyCode::F2) || keys.just_pressed(KeyCode::Backquote) || keys.just_pressed(KeyCode::Slash) || mobile_input.toggle_editor {
        mobile_input.toggle_editor = false;
        editor.0 = !editor.0;
        if editor.0 {
            undo_stack.0.clear();
        } else {
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

pub fn editor_scroll_zoom(
    editor: Res<EditorMode>,
    mut scroll_events: EventReader<MouseWheel>,
    mut camera_query: Query<&mut OrthographicProjection, With<Camera>>,
) {
    if !editor.0 {
        return;
    }
    let Ok(mut projection) = camera_query.get_single_mut() else { return };
    for event in scroll_events.read() {
        let delta = -event.y * 0.02;
        projection.scale = (projection.scale + delta).clamp(0.3, 3.0);
    }
}

pub fn editor_mobile_zoom(
    editor: Res<EditorMode>,
    mut mobile_input: ResMut<MobileInput>,
    mut camera_query: Query<&mut OrthographicProjection, With<Camera>>,
) {
    if !editor.0 {
        return;
    }
    let Ok(mut projection) = camera_query.get_single_mut() else { return };
    if mobile_input.zoom_in {
        mobile_input.zoom_in = false;
        projection.scale = (projection.scale - 0.1).clamp(0.3, 3.0);
    }
    if mobile_input.zoom_out {
        mobile_input.zoom_out = false;
        projection.scale = (projection.scale + 0.1).clamp(0.3, 3.0);
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
    if cursor.x > window.width() - OVERLAY_WIDTH {
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

    transform.translation = Vec3::new(grid_x, grid_y, Z_EDITOR_CURSOR);
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

fn find_safe_spawn(tiles: &[[u8; MAP_WIDTH]; MAP_HEIGHT]) -> GridPos {
    let start_row = 18usize;
    let start_col = 1usize;
    if start_row < MAP_HEIGHT && start_col < MAP_WIDTH
        && tiles[start_row][start_col] != 1
    {
        return GridPos { x: start_col as i32, y: (MAP_HEIGHT as i32 - 1 - start_row as i32) };
    }
    for row in 0..MAP_HEIGHT {
        for col in 0..MAP_WIDTH {
            if tiles[row][col] == 0 {
                return GridPos { x: col as i32, y: (MAP_HEIGHT as i32 - 1 - row as i32) };
            }
        }
    }
    GridPos { x: 1, y: 1 }
}

fn cycle_direction(dir: &Direction) -> Direction {
    match dir {
        Direction::Up => Direction::Right,
        Direction::Right => Direction::Down,
        Direction::Down => Direction::Left,
        Direction::Left => Direction::Up,
    }
}

pub fn editor_place_tile(
    editor: Res<EditorMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    selected: Res<SelectedTile>,
    selected_npc: Res<SelectedNpc>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut level_data: ResMut<LevelData>,
    mut rebuild: ResMut<RebuildRequested>,
    mut undo_stack: ResMut<UndoStack>,
) {
    if !editor.0 {
        return;
    }

    let left = mouse.just_pressed(MouseButton::Left);
    let right = mouse.just_pressed(MouseButton::Right);
    if !left && !right {
        return;
    }

    let window = windows.single();
    let Some(cursor) = window.cursor_position() else { return };

    if cursor.x < PALETTE_WIDTH {
        return;
    }
    if cursor.x > window.width() - OVERLAY_WIDTH {
        return;
    }

    let Ok((camera, camera_transform)) = cameras.get_single() else { return };
    let Some((gx, gy)) = screen_to_grid(cursor, camera, camera_transform) else { return };

    let row = (MAP_HEIGHT as i32 - 1 - gy) as usize;
    let col = gx as usize;
    let pos = GridPos { x: gx, y: gy };

    if let Some(npc_kind) = selected_npc.0 {
        let already_exists = level_data.npcs.iter().any(|n| n.pos == pos);
        if left && already_exists {
            level_data.npcs.retain(|n| n.pos != pos);
            rebuild.0 = true;
        } else if left {
            level_data.npcs.push(crate::resources::NpcSpawnData {
                kind: npc_kind,
                pos,
                facing: Direction::Down,
            });
            rebuild.0 = true;
        } else if right && already_exists {
            level_data.npcs.retain(|n| n.pos != pos);
            rebuild.0 = true;
        }
        return;
    }

    if right {
        let erased_npc = level_data.npcs.iter().any(|n| n.pos == pos);
        if erased_npc {
            level_data.npcs.retain(|n| n.pos != pos);
            rebuild.0 = true;
            return;
        }
        let old_tile = level_data.tiles[row][col];
        if old_tile != 0 || selected.0 != 0 {
            undo_stack.0.push(UndoEntry { row, col, old_tile, old_dir: None });
        }
        level_data.tiles[row][col] = 0;
        level_data.conveyor_dirs.remove(&(col, row));
        rebuild.0 = true;
        return;
    }

    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let old_tile = level_data.tiles[row][col];

    if shift && old_tile == CONVEYOR {
        let dir = level_data.conveyor_dirs
            .entry((col, row))
            .or_insert(Direction::Down);
        *dir = cycle_direction(dir);
        rebuild.0 = true;
        return;
    }

    if old_tile != selected.0 {
        undo_stack.0.push(UndoEntry {
            row, col, old_tile,
            old_dir: if old_tile == CONVEYOR { level_data.conveyor_dirs.get(&(col, row)).copied() } else { None },
        });
        if undo_stack.0.len() > 100 {
            undo_stack.0.remove(0);
        }
    }

    level_data.tiles[row][col] = selected.0;

    if selected.0 == CONVEYOR {
        level_data.conveyor_dirs.entry((col, row)).or_insert(Direction::Down);
    } else {
        level_data.conveyor_dirs.remove(&(col, row));
    }

    rebuild.0 = true;
}

pub fn editor_undo(
    editor: Res<EditorMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mut level_data: ResMut<LevelData>,
    mut undo_stack: ResMut<UndoStack>,
    mut rebuild: ResMut<RebuildRequested>,
) {
    if !editor.0 {
        return;
    }
    if keys.just_pressed(KeyCode::KeyZ) && (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)) {
        if let Some(entry) = undo_stack.0.pop() {
            level_data.tiles[entry.row][entry.col] = entry.old_tile;
            match entry.old_dir {
                Some(dir) => { level_data.conveyor_dirs.insert((entry.col, entry.row), dir); }
                None => { level_data.conveyor_dirs.remove(&(entry.col, entry.row)); }
            }
            rebuild.0 = true;
        }
    }
}

pub fn handle_palette_buttons(
    mut selected_tile: ResMut<SelectedTile>,
    mut selected_npc: ResMut<SelectedNpc>,
    tile_buttons: Query<(&Interaction, &TileButton), Changed<Interaction>>,
    npc_buttons: Query<(&Interaction, &NpcPaletteButton), Changed<Interaction>>,
) {
    for (interaction, button) in tile_buttons.iter() {
        if *interaction == Interaction::Pressed {
            selected_tile.0 = button.0;
            selected_npc.0 = None;
        }
    }
    for (interaction, button) in npc_buttons.iter() {
        if *interaction == Interaction::Pressed {
            selected_npc.0 = Some(button.0);
            selected_tile.0 = 0;
        }
    }
}

pub fn handle_editor_save_load(
    save_buttons: Query<&Interaction, (With<EditorSaveButton>, Changed<Interaction>)>,
    load_buttons: Query<&Interaction, (With<EditorLoadButton>, Changed<Interaction>)>,
    reset_buttons: Query<&Interaction, (With<EditorResetButton>, Changed<Interaction>)>,
    mut level_data: ResMut<LevelData>,
    mut rebuild: ResMut<RebuildRequested>,
) {
    if save_buttons.iter().any(|i| *i == Interaction::Pressed) {
        crate::io::save_level_data(&level_data);
    }
    if load_buttons.iter().any(|i| *i == Interaction::Pressed) {
        if let Some(loaded) = crate::io::load_level_data() {
            *level_data = loaded;
            rebuild.0 = true;
        }
    }
    if reset_buttons.iter().any(|i| *i == Interaction::Pressed) {
        crate::io::delete_saved_level();
        *level_data = LevelData::default_level();
        rebuild.0 = true;
    }
}

pub fn update_palette_highlight(
    selected_tile: Res<SelectedTile>,
    selected_npc: Res<SelectedNpc>,
    mut tile_buttons: Query<(&TileButton, &mut BackgroundColor), Without<NpcPaletteButton>>,
    mut npc_buttons: Query<(&NpcPaletteButton, &mut BackgroundColor)>,
) {
    for (button, mut bg) in tile_buttons.iter_mut() {
        bg.0 = if button.0 == selected_tile.0 && selected_npc.0.is_none() {
            Color::WHITE
        } else {
            button.1
        };
    }
    for (button, mut bg) in npc_buttons.iter_mut() {
        bg.0 = if selected_npc.0 == Some(button.0) {
            Color::WHITE
        } else {
            button.1
        };
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

const TILE_KEY_MAP: &[(KeyCode, u8)] = &[
    (KeyCode::Digit0, 0),
    (KeyCode::Digit1, 1),
    (KeyCode::Digit2, 2),
    (KeyCode::Digit3, 3),
    (KeyCode::Digit4, 4),
    (KeyCode::Digit5, 5),
    (KeyCode::Digit6, 6),
    (KeyCode::Digit7, 7),
    (KeyCode::Digit8, 8),
];

const NPC_KEY_MAP: &[(KeyCode, NpcKind)] = &[
    (KeyCode::F9, NpcKind::ConveyorLoader),
    (KeyCode::F10, NpcKind::OvenHauler),
    (KeyCode::F11, NpcKind::PackerHauler),
];

pub fn editor_palette_keyboard(
    editor: Res<EditorMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedTile>,
    mut selected_npc: ResMut<SelectedNpc>,
) {
    if !editor.0 {
        return;
    }

    for &(key, tile) in TILE_KEY_MAP {
        if keys.just_pressed(key) {
            selected.0 = tile;
            selected_npc.0 = None;
            return;
        }
    }

    for &(key, kind) in NPC_KEY_MAP {
        if keys.just_pressed(key) {
            selected_npc.0 = Some(kind);
            selected.0 = 0;
            return;
        }
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

    let row = (MAP_HEIGHT as i32 - 1 - player_pos.y) as usize;
    let col = player_pos.x as usize;
    let is_blocked = if row < MAP_HEIGHT && col < MAP_WIDTH {
        level_data.tiles[row][col] == 1 || level_data.tiles[row][col] == CONVEYOR
    } else {
        false
    };
    let safe_player_pos = if is_blocked {
        find_safe_spawn(&level_data.tiles)
    } else {
        player_pos
    };

    let entities: Vec<Entity> = game_entities.iter().collect();
    for entity in entities {
        commands.entity(entity).despawn();
    }

    crate::level::setup_level(&mut commands, &level_data);
    crate::player::spawn_player_at(&mut commands, safe_player_pos);
    for npc_data in &level_data.npcs {
        crate::npc::spawn_npc_from_data(&mut commands, npc_data);
    }
    crate::ui::setup_ui(&mut commands);
}

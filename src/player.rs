use bevy::prelude::*;
use crate::components::*;
use crate::level::*;
use crate::level::{TILE_SIZE, MAP_HEIGHT, PLAYER_START, Z_PLAYER, Z_PLAYER_INDICATOR, INDICATOR_HALF, INDICATOR_BAR_HALF};

pub fn spawn_player(commands: &mut Commands) {
    let (gx, row) = PLAYER_START;
    let gy = (MAP_HEIGHT - 1 - row) as i32;
    let pos = GridPos { x: gx as i32, y: gy };
    spawn_player_at(commands, pos);
}

pub fn spawn_player_at(commands: &mut Commands, pos: GridPos) {
    let world_pos = grid_to_world(pos);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.3, 0.6, 1.0),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.7, TILE_SIZE * 0.7)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                world_pos.x,
                world_pos.y,
                Z_PLAYER,
            )),
            visibility: Visibility::Visible,
            ..default()
        },
        Player,
        Facing(crate::components::Direction::Up),
        Carrying::empty(),
        pos,
        GameEntity,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.6, 0.8, 1.0),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.7, TILE_SIZE * 0.15)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                world_pos.x,
                world_pos.y + TILE_SIZE * 0.35 - TILE_SIZE * 0.075,
                Z_PLAYER_INDICATOR,
            )),
            ..default()
        },
        DirectionIndicator,
        GameEntity,
    ));
}

pub fn update_direction_indicator(
    player_query: Query<(&Transform, &Facing), (With<Player>, Without<DirectionIndicator>)>,
    mut indicator_query: Query<(&mut Transform, &mut Sprite), (With<DirectionIndicator>, Without<Player>)>,
) {
    let Ok((player_transform, facing)) = player_query.get_single() else { return };
    let Ok((mut transform, mut sprite)) = indicator_query.get_single_mut() else { return };

    let (offset_x, offset_y, width, height) = facing.0.indicator_offset(
        INDICATOR_HALF,
        INDICATOR_BAR_HALF,
    );
    transform.translation = Vec3::new(
        player_transform.translation.x + offset_x,
        player_transform.translation.y + offset_y,
        Z_PLAYER_INDICATOR,
    );
    sprite.custom_size = Some(Vec2::new(width, height));
}

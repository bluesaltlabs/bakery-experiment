use bevy::prelude::*;
use crate::components::*;
use crate::level::*;

pub fn spawn_player(commands: &mut Commands) {
    let (gx, row) = PLAYER_START;
    let gy = (MAP_HEIGHT - 1 - row) as i32;
    let pos = GridPos { x: gx as i32, y: gy };

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.3, 0.6, 1.0),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.7, TILE_SIZE * 0.7)),
                ..default()
            },
            transform: Transform::from_translation(grid_to_world(pos)),
            ..default()
        },
        Player,
        Facing(crate::components::Direction::Up),
        Carrying(None),
        pos,
        GameEntity,
    ));
}

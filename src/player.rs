use bevy::prelude::*;
use crate::agent;
use crate::components::*;
use crate::level::*;

pub fn spawn_player(commands: &mut Commands) {
    let (gx, row) = PLAYER_START;
    let gy = (MAP_HEIGHT - 1 - row) as i32;
    let pos = GridPos { x: gx as i32, y: gy };
    spawn_player_at(commands, pos);
}

pub fn spawn_player_at(commands: &mut Commands, pos: GridPos) {
    let world_pos = grid_to_world(pos);

    let entity = commands.spawn((
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
        agent::AgentBundle {
            grid_pos: pos,
            facing: Facing(crate::components::Direction::Up),
            carrying: Carrying::empty(),
            game_entity: GameEntity,
        },
        Player,
    )).id();

    agent::spawn_indicator(commands, entity, world_pos, Color::srgb(0.6, 0.8, 1.0));
}

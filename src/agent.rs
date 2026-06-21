use bevy::prelude::*;
use crate::components::*;

#[derive(Bundle)]
pub struct AgentBundle {
    pub grid_pos: GridPos,
    pub facing: Facing,
    pub carrying: Carrying,
    pub game_entity: GameEntity,
}

pub fn spawn_indicator(
    commands: &mut Commands,
    parent: Entity,
    world_pos: Vec3,
    color: Color,
) -> Entity {
    let ts = crate::level::TILE_SIZE;
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(ts * 0.7, ts * 0.15)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                world_pos.x,
                world_pos.y + ts * 0.35 - ts * 0.075,
                crate::level::Z_PLAYER_INDICATOR,
            )),
            ..default()
        },
        DirectionIndicator { parent },
        GameEntity,
    )).id()
}

pub fn update_direction_indicators(
    parent_query: Query<(Entity, &Transform, &Facing), (Or<(With<Player>, With<Npc>)>, Without<DirectionIndicator>)>,
    mut indicator_query: Query<(&DirectionIndicator, &mut Transform, &mut Sprite)>,
) {
    for (parent_entity, parent_transform, facing) in parent_query.iter() {
        for (indicator, mut transform, mut sprite) in indicator_query.iter_mut() {
            if indicator.parent != parent_entity {
                continue;
            }
            let (ox, oy, w, h) = facing.0.indicator_offset(
                crate::level::INDICATOR_HALF,
                crate::level::INDICATOR_BAR_HALF,
            );
            transform.translation = Vec3::new(
                parent_transform.translation.x + ox,
                parent_transform.translation.y + oy,
                crate::level::Z_PLAYER_INDICATOR,
            );
            sprite.custom_size = Some(Vec2::new(w, h));
        }
    }
}

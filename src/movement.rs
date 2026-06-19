use bevy::prelude::*;
use crate::audio::AudioEventQueue;
use crate::components::{ConveyorBelt, Direction, Facing, GridPos, Npc, Player, Solid, Station};
use crate::level::grid_to_world;
use crate::mobile::MobileInput;
use crate::resources::MovementCooldown;

pub fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut cooldown: ResMut<MovementCooldown>,
    mut player_query: Query<(&mut GridPos, &mut Transform, &mut Facing), With<Player>>,
    solid_query: Query<&GridPos, (With<Solid>, Without<Player>)>,
    station_query: Query<&GridPos, (With<Station>, Without<Player>)>,
    conveyor_query: Query<&GridPos, (With<ConveyorBelt>, Without<Player>)>,
    npc_query: Query<&GridPos, (With<Npc>, Without<Player>)>,
    mut audio_queue: ResMut<AudioEventQueue>,
    mut mobile_input: ResMut<MobileInput>,
) {
    cooldown.0.tick(time.delta());
    if !cooldown.0.finished() {
        return;
    }

    let dir = if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        Some(Direction::Up)
    } else if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        Some(Direction::Down)
    } else if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        Some(Direction::Left)
    } else if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        Some(Direction::Right)
    } else if let Some(dir) = mobile_input.direction.take() {
        Some(dir)
    } else {
        None
    };

    let Some(dir) = dir else { return };

    let (mut pos, mut transform, mut facing) = player_query.single_mut();
    facing.0 = dir;

    let delta = dir.delta();
    let new_pos = GridPos {
        x: pos.x + delta.0,
        y: pos.y + delta.1,
    };

    let blocked = solid_query.iter().any(|gp| *gp == new_pos)
        || station_query.iter().any(|gp| *gp == new_pos)
        || conveyor_query.iter().any(|gp| *gp == new_pos)
        || npc_query.iter().any(|gp| *gp == new_pos);

    if !blocked {
        pos.x = new_pos.x;
        pos.y = new_pos.y;
        transform.translation = grid_to_world(new_pos);
        transform.translation.z = 0.01;
        audio_queue.0.push(crate::audio::AudioEvent::Step);
    }

    cooldown.0.reset();
}

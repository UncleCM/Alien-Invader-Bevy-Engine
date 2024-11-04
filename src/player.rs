use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use crate::projectile::Projectile;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerMovement {
    pub speed: f32,
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_texture = asset_server.load("player.png");

    commands.spawn(SpriteBundle {
        texture: player_texture,
        transform: Transform {
            translation: Vec3::new(0.0, -250.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(Player)
    .insert(PlayerMovement { speed: 300.0 });
}

pub fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&PlayerMovement, &mut Transform), With<Player>>,
) {
    for (movement, mut transform) in query.iter_mut() {
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::Left) {
            direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            direction += 1.0;
        }

        transform.translation.x += direction * movement.speed * time.delta_seconds();

        let screen_half_width = 400.0;
        transform.translation.x = transform.translation.x.clamp(-screen_half_width, screen_half_width);
    }
}

pub fn spawn_projectile(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Transform, With<Player>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(player_transform) = query.single() {
            let projectile_texture = asset_server.load("projectile.png");

            commands.spawn(SpriteBundle {
                texture: projectile_texture,
                transform: Transform {
                    translation: player_transform.translation,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Projectile { speed: 500.0 });
        }
    }
}

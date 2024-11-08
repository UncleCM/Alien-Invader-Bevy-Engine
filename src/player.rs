use bevy::prelude::*;

use crate::resolution;
use crate::projectile;
use crate::gyro_hander::InputState;  // Update this import

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
           .add_systems(Update, update_player);
    }
}

#[derive(Component)]
pub struct Player {
    pub shoot_timer: f32,
}

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    resolution: Res<resolution::Resolution>,
) {
    let player_image = asset_server.load("player.png");
    commands.spawn((
        SpriteBundle {
            texture: player_image,
            transform: Transform::from_xyz(
                0., 
                -(resolution.screen_dimensions.y * 0.5) + (resolution.pixel_ratio * 5.0),
                0.
            ).with_scale(Vec3::splat(resolution.pixel_ratio)),
            ..Default::default()
        },
        Player { shoot_timer: 0. }
    ));
}

const SPEED: f32 = 200.;
const BULLET_SPEED: f32 = 400.;
const SHOOT_COOLDOWN: f32 = 0.5;
const TILT_THRESHOLD: f32 = 15.0; // Degrees of tilt needed to trigger shooting

fn update_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    time: Res<Time>,
    input_state: Res<InputState>,  // Use InputState instead of GyroState
    resolution: Res<resolution::Resolution>,
) {
    let (mut player, mut transform) = player_query.single_mut();

    // Use horizontal movement from InputState
    let horizontal = input_state.horizontal_movement;
    
    // Create movement vector
    let direction = Vec3::new(horizontal, 0.0, 0.0).normalize_or_zero();
    
    // Move player
    transform.translation += direction * SPEED * time.delta_seconds();
    
    // Confine player movement
    let left_bound = -resolution.screen_dimensions.x * 0.5;
    let right_bound = resolution.screen_dimensions.x * 0.5;
    
    transform.translation.x = transform.translation.x.clamp(left_bound, right_bound);
    
    // Update shoot timer
    player.shoot_timer -= time.delta_seconds();
    
    // Trigger shooting based on shoot_requested from InputState
    if input_state.shoot_requested && player.shoot_timer <= 0. {
        player.shoot_timer = SHOOT_COOLDOWN;
        let bullet_texture = asset_server.load("bullet.png");
        commands.spawn((
            SpriteBundle {
                texture: bullet_texture,
                transform: Transform::from_translation(transform.translation)
                    .with_scale(Vec3::splat(resolution.pixel_ratio)),
                ..Default::default()
            },
            projectile::Projectile {
                speed: BULLET_SPEED,
            },
        ));
    }
}
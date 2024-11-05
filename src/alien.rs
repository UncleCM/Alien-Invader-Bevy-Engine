use bevy::prelude::*;
use rand::Rng; // For generating random spawn positions

use crate::resolution;

pub struct AlienPlugin;

impl Plugin for AlienPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_aliens)
            .add_systems(Update, (update_aliens, manage_alien_logic));
    }
}

#[derive(Component)]
pub struct Alien {
    pub dead: bool,
    pub original_position: Vec3,
}

#[derive(Component)]
pub struct Dead;

#[derive(Resource)]
pub struct AlienManager {
    pub direction: f32,
    pub shift_aliens_down: bool,
    pub dist_from_boundary: f32,
    pub reset: bool,
}

// Add a Player component to track the player’s position.
#[derive(Component)]
pub struct Player;

const NUM_ALIENS: i32 = 10;
const SPEED: f32 = 100.0;

fn setup_aliens(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    resolution: Res<resolution::Resolution>,
) {
    commands.insert_resource(AlienManager {
        reset: false,
        dist_from_boundary: 0.,
        shift_aliens_down: false,
        direction: 1.,
    });

    // Load the alien texture
    let alien_texture = asset_server.load("alien.png");

    let mut rng = rand::thread_rng();
    for _ in 0..NUM_ALIENS {
        let x = rng.gen_range(-resolution.screen_dimensions.x * 0.5..resolution.screen_dimensions.x * 0.5);
        let y = rng.gen_range(0.0..resolution.screen_dimensions.y * 0.5);
        let position = Vec3::new(x, y, 0.);

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(position).with_scale(Vec3::splat(resolution.pixel_ratio)),
                texture: alien_texture.clone(),
                ..default()
            },
            Alien {
                original_position: position,
                dead: false,
            },
        ));
    }

    // Spawn player at the center
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0.0, -resolution.screen_dimensions.y * 0.4, 0.)),
            ..default()
        },
        Player,
    ));
}

fn update_aliens(
    mut commands: Commands,
    mut query: ParamSet<(
        Query<(Entity, &Alien, &mut Transform, &mut Visibility), Without<Dead>>,
        Query<&Transform, With<Player>>,
    )>,
    mut alien_manager: ResMut<AlienManager>,
    resolution: Res<resolution::Resolution>,
    time: Res<Time>,
) {
    // Create a longer-lived reference for the player's Transform
    let player_transform = query.p1().single().clone(); // Clone if necessary

    // Iterate over aliens and update their position
    for (entity, alien, mut transform, mut visibility) in query.p0().iter_mut() {
        let direction_to_player = (player_transform.translation - transform.translation).normalize();
        transform.translation += direction_to_player * SPEED * time.delta_seconds();

        if alien.dead {
            commands.entity(entity).insert(Dead {});
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }

        if transform.translation.y < -resolution.screen_dimensions.y * 0.5 {
            alien_manager.reset = true;
        }
    }
}

fn manage_alien_logic(
    mut commands: Commands,
    mut alien_query: Query<(Entity, &mut Alien, &mut Transform)>,
    mut alien_manager: ResMut<AlienManager>,
) {
    if alien_manager.reset {
        alien_manager.reset = false;
        alien_manager.direction = 1.;

        for (entity, mut alien, mut transform) in alien_query.iter_mut() {
            transform.translation = alien.original_position;
            if alien.dead {
                alien.dead = false;
                commands.entity(entity).remove::<Dead>();
            }
        }
    }
}

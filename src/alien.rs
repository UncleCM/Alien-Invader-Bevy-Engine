use bevy::prelude::*;
use crate::resolution;
use rand::Rng;
use crate::player::Player;
use crate::resolution::Resolution;

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
    pub velocity: Vec2,
}

#[derive(Component)]
pub struct Dead;

#[derive(Resource)]
pub struct AlienManager {
    pub reset: bool,
    pub alive_aliens: usize,
}

pub const WIDTH: i32 = 5;
pub const HEIGHT: i32 = 2;
const SPACING: f32 = 24.;
const ALIEN_SPEED: f32 = 50.0;
const ALIEN_VELOCITY_RANGE: (f32, f32) = (-10.0, 10.0);

fn setup_aliens(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    resolution: Res<resolution::Resolution>,
    alien_query: Query<&Transform, With<Alien>>,
) {
    commands.insert_resource(AlienManager {
        reset: false,
        alive_aliens: (WIDTH * HEIGHT) as usize,
    });
    let alien_texture = asset_server.load("alien.png");
    let mut rng = rand::thread_rng();
    for _ in 0..(WIDTH * HEIGHT) {
        let mut x;
        let mut y;
        let mut position_valid;
        loop {
            x = rng.gen_range(-resolution.screen_dimensions.x * 0.5..resolution.screen_dimensions.x * 0.5);
            y = resolution.screen_dimensions.y * 0.5 - SPACING;
            position_valid = true;
            for transform in alien_query.iter() {
                if Vec2::distance(Vec2::new(x, y), Vec2::new(transform.translation.x, transform.translation.y)) < SPACING {
                    position_valid = false;
                    break;
                }
            }
            if position_valid {
                break;
            }
        }
        let velocity_x = rng.gen_range(ALIEN_VELOCITY_RANGE.0..ALIEN_VELOCITY_RANGE.1);
        let velocity_y = rng.gen_range(ALIEN_VELOCITY_RANGE.0..ALIEN_VELOCITY_RANGE.1);
        commands.spawn((
            SpriteBundle {
                texture: alien_texture.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y, 0.0),
                    scale: Vec3::splat(2.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            Alien {
                dead: false,
                original_position: Vec3::new(x, y, 0.0),
                velocity: Vec2::new(velocity_x, velocity_y),
            },
        ));
    }
}

fn update_aliens(
    mut commands: Commands,
    mut param_set: ParamSet<(
        Query<(Entity, &mut Alien, &mut Transform, &mut Visibility)>,
        Query<&Transform, With<Player>>,
    )>,
    mut alien_manager: ResMut<AlienManager>,
    resolution: Res<resolution::Resolution>,
    time: Res<Time>,
) {
    // Create a longer-lived binding for the player's transform
    let binding = param_set.p1();
    let player_transform = binding.single();
    let player_position = player_transform.translation;

    for (entity, mut alien, mut transform, _visibility) in param_set.p0().iter_mut() {
        // Calculate the direction vector from the alien to the player
        let direction = (player_position - transform.translation).normalize();

        // Update the alien's velocity to move towards the player
        alien.velocity = Vec2::new(direction.x, direction.y) * ALIEN_SPEED;

        // Move the alien
        transform.translation += Vec3::new(
            alien.velocity.x * time.delta_seconds(),
            alien.velocity.y * time.delta_seconds(),
            0.,
        );

        // Check if the alien is out of bounds and mark it as dead
        if transform.translation.x < -resolution.screen_dimensions.x * 0.5
        || transform.translation.x > resolution.screen_dimensions.x * 0.5
        || transform.translation.y < -resolution.screen_dimensions.y * 0.5
        || transform.translation.y > resolution.screen_dimensions.y * 0.5
        {
            alien.dead = true;
            println!("Alien went out of bounds and is now dead.");
        }

        // If the alien is dead, despawn it and update the alive_aliens count
        if alien.dead {
            commands.entity(entity).despawn();
            if alien_manager.alive_aliens > 0 {
                alien_manager.alive_aliens -= 1;
                println!("Alien despawned. Alive aliens count: {}", alien_manager.alive_aliens);
            }
        }
    }
}

fn manage_alien_logic(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut alien_query: Query<(Entity, &mut Alien, &mut Transform)>,
    mut alien_manager: ResMut<AlienManager>,
    resolution: Res<Resolution>,
) {
    if alien_manager.alive_aliens == 0 {
        // Respawn all aliens
        let mut alive_count = 0;
        for (entity, mut alien, mut transform) in alien_query.iter_mut() {
            if alien.dead {
                transform.translation = alien.original_position;
                alien.dead = false;
                commands.entity(entity).remove::<Dead>();
                alive_count += 1;
                println!("Respawning alien at position: {:?}", alien.original_position);
            }
        }
        alien_manager.alive_aliens = alive_count;
        println!("All aliens respawned. Alive aliens count: {}", alien_manager.alive_aliens);
    }
}

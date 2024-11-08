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
    // Initialize the AlienManager resource
    commands.insert_resource(AlienManager {
        reset: false,
        alive_aliens: (WIDTH * HEIGHT) as usize,
    });

    // Load the alien texture
    let alien_texture = asset_server.load("alien.png");
    let mut rng = rand::thread_rng();

    // Spawn aliens in a grid, ensuring they don't overlap
    for _ in 0..(WIDTH * HEIGHT) {
        let mut x;
        let mut y;
        let mut position_valid;

        // Keep trying until a valid position is found
        loop {
            x = rng.gen_range(-resolution.screen_dimensions.x * 0.5..resolution.screen_dimensions.x * 0.5);
            y = resolution.screen_dimensions.y * 0.5 - SPACING;
            position_valid = true;

            // Check if any other alien is too close to the new position
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

        // Randomize alien velocity
        let velocity_x = rng.gen_range(ALIEN_VELOCITY_RANGE.0..ALIEN_VELOCITY_RANGE.1);
        let velocity_y = rng.gen_range(ALIEN_VELOCITY_RANGE.0..ALIEN_VELOCITY_RANGE.1);

        // Spawn the alien
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
    let binding = param_set.p1();
    let player_transform = binding.single();
    let player_position = player_transform.translation;

    for (entity, mut alien, mut transform, _visibility) in param_set.p0().iter_mut() {
        // Move alien towards player
        let direction = (player_position - transform.translation).normalize();
        alien.velocity = Vec2::new(direction.x, direction.y) * ALIEN_SPEED;

        // Apply velocity to alien position
        transform.translation += Vec3::new(
            alien.velocity.x * time.delta_seconds(),
            alien.velocity.y * time.delta_seconds(),
            0.0,
        );

        // Check if alien is out of bounds
        let alien_position = Vec2::new(transform.translation.x, transform.translation.y);
        let screen_half_x = resolution.screen_dimensions.x * 0.5;
        let screen_half_y = resolution.screen_dimensions.y * 0.5;

        if alien_position.x < -screen_half_x || alien_position.x > screen_half_x
            || alien_position.y < -screen_half_y || alien_position.y > screen_half_y
        {
            alien.dead = true;
            println!("Alien went out of bounds and is now dead.");
        }

        // If the alien is dead, despawn it and update the alive count
        if alien.dead {
            commands.entity(entity).despawn();
            if alien_manager.alive_aliens > 0 {
                alien_manager.alive_aliens -= 1;
                println!("Alien despawned. Alive aliens count: {}", alien_manager.alive_aliens);
            } else {
                println!("No more alive aliens to despawn.");
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
    time: Res<Time>,
) {
    // Calculate how many aliens are alive before any respawn logic
    let mut alive_count = 0;

    for (_, alien, _) in alien_query.iter_mut() {
        if !alien.dead {
            alive_count += 1;
        }
    }

    // Update the alive_aliens count in the manager
    alien_manager.alive_aliens = alive_count;

    // When no aliens are left, respawn them and create new aliens
    if alien_manager.alive_aliens == 0 {
        println!("All aliens are dead, spawning new aliens.");

        // Number of aliens to respawn + random extra aliens
        let base_count = WIDTH * HEIGHT; // original number of aliens
        let extra_aliens = rand::thread_rng().gen_range(1..=3); // Random additional aliens

        let total_aliens_to_spawn = base_count + extra_aliens;

        // Respawn all dead aliens
        for (entity, mut alien, mut transform) in alien_query.iter_mut() {
            if alien.dead {
                // Respawn the alien at its original position
                transform.translation = alien.original_position;
                alien.velocity = Vec2::new(0.0, 0.0); // Reset velocity or keep original
                alien.dead = false; // Mark as alive
                println!("Respawned alien at position: {:?}", alien.original_position);
            }
        }

        // After respawning aliens, check if any aliens are alive
        alive_count = alien_query.iter().filter(|(_, alien, _)| !alien.dead).count();

        // If no aliens are alive after respawning, spawn the random number of new aliens
        if alive_count == 0 {
            println!("No aliens respawned, spawning {} new aliens.", total_aliens_to_spawn);
            for _ in 0..total_aliens_to_spawn {
                spawn_new_alien(&mut commands, &asset_server, &resolution);
            }
        }

        // Update the number of alive aliens after respawning
        alien_manager.alive_aliens = alive_count + (extra_aliens as usize); // Add the extra aliens to the count
        println!("Alive aliens count after respawn: {}", alien_manager.alive_aliens);
    }
}



/// Function to spawn a new alien when all are dead
fn spawn_new_alien(
    commands: &mut Commands,
    asset_server: &AssetServer,
    resolution: &Resolution,
) {
    let alien_texture = asset_server.load("alien.png");
    let mut rng = rand::thread_rng();

    // Generate random position for new alien
    let x = rng.gen_range(-resolution.screen_dimensions.x * 0.5..resolution.screen_dimensions.x * 0.5);
    let y = resolution.screen_dimensions.y * 0.5 - SPACING;

    // Randomize alien velocity
    let velocity_x = rng.gen_range(ALIEN_VELOCITY_RANGE.0..ALIEN_VELOCITY_RANGE.1);
    let velocity_y = rng.gen_range(ALIEN_VELOCITY_RANGE.0..ALIEN_VELOCITY_RANGE.1);

    // Spawn the new alien
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

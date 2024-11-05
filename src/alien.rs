use bevy::prelude::*;
use rand::prelude::*;
use crate::resolution;
use crate::projectile;

pub struct AlienPlugin;

impl Plugin for AlienPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SpawnTimer(Timer::from_seconds(3.0, TimerMode::Repeating)))
            .add_systems(Startup, setup_alien_manager)
            .add_systems(Update, (
                spawn_aliens,
                update_aliens,
                manage_alien_logic,
                alien_shooting,
            ));
    }
}

#[derive(Component)]
pub struct Alien {
    pub dead: bool,
    pub original_position: Vec3,
    pub movement_timer: Timer,
    pub shoot_timer: Timer,
    pub movement_pattern: MovementPattern,
}

#[derive(Component)]
pub struct Dead;

#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Resource)]
pub struct AlienManager {
    pub active_aliens: usize,
    pub max_aliens: usize,
}

#[derive(Component)]
enum MovementPattern {
    Zigzag { amplitude: f32, frequency: f32, phase: f32 },
    Circle { radius: f32, speed: f32, center: Vec2 },
    Linear { direction: Vec2 },
}

const SPEED: f32 = 100.0;
const MAX_ALIENS: usize = 15;
const SHOOT_INTERVAL: f32 = 2.0;
const MOVEMENT_CHANGE_INTERVAL: f32 = 3.0;

fn setup_alien_manager(mut commands: Commands) {
    commands.insert_resource(AlienManager {
        active_aliens: 0,
        max_aliens: MAX_ALIENS,
    });
}

fn spawn_aliens(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut spawn_timer: ResMut<SpawnTimer>,
    mut alien_manager: ResMut<AlienManager>,
    resolution: Res<resolution::Resolution>,
) {
    if spawn_timer.0.tick(time.delta()).just_finished() && alien_manager.active_aliens < alien_manager.max_aliens {
        let mut rng = rand::thread_rng();
        
        // Random spawn position at the top of the screen
        let x = rng.gen_range(-resolution.screen_dimensions.x * 0.4..resolution.screen_dimensions.x * 0.4);
        let y = resolution.screen_dimensions.y * 0.5;
        let position = Vec3::new(x, y, 0.0);

        // Random movement pattern
        let movement_pattern = match rng.gen_range(0..3) {
            0 => MovementPattern::Zigzag {
                amplitude: 100.0,
                frequency: 1.0,
                phase: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
            },
            1 => MovementPattern::Circle {
                radius: 50.0,
                speed: 1.0,
                center: Vec2::new(x, y - 50.0),
            },
            _ => MovementPattern::Linear {
                direction: Vec2::new(rng.gen_range(-1.0..1.0), -1.0).normalize(),
            },
        };

        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("alien.png"),
                transform: Transform::from_translation(position)
                    .with_scale(Vec3::splat(resolution.pixel_ratio)),
                ..default()
            },
            Alien {
                dead: false,
                original_position: position,
                movement_timer: Timer::from_seconds(MOVEMENT_CHANGE_INTERVAL, TimerMode::Repeating),
                shoot_timer: Timer::from_seconds(
                    rng.gen_range(SHOOT_INTERVAL..SHOOT_INTERVAL * 2.0),
                    TimerMode::Repeating,
                ),
                movement_pattern,
            },
        ));

        alien_manager.active_aliens += 1;
    }
}

fn update_aliens(
    mut commands: Commands,
    time: Res<Time>,
    resolution: Res<resolution::Resolution>,
    mut alien_query: Query<(Entity, &mut Alien, &mut Transform), Without<Dead>>,
    mut alien_manager: ResMut<AlienManager>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut alien, mut transform) in alien_query.iter_mut() {
        if alien.dead {
            commands.entity(entity).insert(Dead);
            alien_manager.active_aliens -= 1;
            continue;
        }

        // Update movement based on pattern
        match &alien.movement_pattern {
            MovementPattern::Zigzag { amplitude, frequency, phase } => {
                let time_offset = time.elapsed_seconds() + phase;
                transform.translation.x += amplitude * (time_offset * frequency).sin() * time.delta_seconds();
                transform.translation.y -= SPEED * 0.5 * time.delta_seconds();
            }
            MovementPattern::Circle { radius, speed, center } => {
                let angle = time.elapsed_seconds() * speed;
                transform.translation.x = center.x + radius * angle.cos();
                transform.translation.y = center.y + radius * angle.sin() - SPEED * 0.3 * time.delta_seconds();
            }
            MovementPattern::Linear { direction } => {
                transform.translation.x += direction.x * SPEED * time.delta_seconds();
                transform.translation.y += direction.y * SPEED * time.delta_seconds();
            }
        }

        // Change movement pattern periodically
        if alien.movement_timer.tick(time.delta()).just_finished() {
            alien.movement_pattern = match rng.gen_range(0..3) {
                0 => MovementPattern::Zigzag {
                    amplitude: 100.0,
                    frequency: 1.0,
                    phase: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
                },
                1 => MovementPattern::Circle {
                    radius: 50.0,
                    speed: 1.0,
                    center: Vec2::new(transform.translation.x, transform.translation.y),
                },
                _ => MovementPattern::Linear {
                    direction: Vec2::new(rng.gen_range(-1.0..1.0), -1.0).normalize(),
                },
            };
        }

        // Remove aliens that go off screen
        if transform.translation.y < -resolution.screen_dimensions.y * 0.5 
            || transform.translation.x.abs() > resolution.screen_dimensions.x * 0.5 {
            commands.entity(entity).despawn();
            alien_manager.active_aliens -= 1;
        }
    }
}

fn alien_shooting(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    resolution: Res<resolution::Resolution>,
    mut alien_query: Query<(&mut Alien, &Transform), Without<Dead>>,
) {
    for (mut alien, transform) in alien_query.iter_mut() {
        if alien.shoot_timer.tick(time.delta()).just_finished() {
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("bullet.png"),
                    transform: Transform::from_translation(transform.translation)
                        .with_scale(Vec3::splat(resolution.pixel_ratio)),
                    ..default()
                },
                projectile::Projectile {
                    speed: -200.0, // Negative speed for downward movement
                },
            ));
        }
    }
}

fn manage_alien_logic(
    mut commands: Commands,
    mut alien_query: Query<(Entity, &mut Alien, &mut Transform)>,
) {
    for (entity, mut alien, mut transform) in alien_query.iter_mut() {
        if alien.dead {
            commands.entity(entity).despawn();
        }
    }
}
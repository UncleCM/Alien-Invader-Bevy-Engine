use bevy::prelude::*;
use crate::resolution;
use rand::Rng;

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

const WIDTH: i32 = 10;
const HEIGHT: i32 = 5;
const SPACING: f32 = 24.;
const ALIEN_SPEED: f32 = 100.0;
const ALIEN_VELOCITY_RANGE: (f32, f32) = (-200.0, 200.0);

fn setup_aliens(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    resolution: Res<resolution::Resolution>,
) {
    commands.insert_resource(AlienManager {
        reset: false,
        alive_aliens: (WIDTH * HEIGHT) as usize,
    });

    let alien_texture = asset_server.load("alien.png");
    let mut rng = rand::thread_rng();

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let position = Vec3::new(
                x as f32 * SPACING - resolution.screen_dimensions.x * 0.5 + SPACING,
                resolution.screen_dimensions.y * 0.5 - SPACING * (y as f32 + 1.),
                0.,
            );

            let velocity = Vec2::new(
                rng.gen_range(ALIEN_VELOCITY_RANGE.0..ALIEN_VELOCITY_RANGE.1),
                rng.gen_range(ALIEN_VELOCITY_RANGE.0..ALIEN_VELOCITY_RANGE.1),
            );

            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_translation(position).with_scale(Vec3::splat(resolution.pixel_ratio)),
                    texture: alien_texture.clone(),
                    ..default()
                },
                Alien {
                    original_position: position,
                    dead: false,
                    velocity,
                },
            ));
        }
    }
}

fn update_aliens(
    mut commands: Commands,
    mut alien_query: Query<(Entity, &mut Alien, &mut Transform, &mut Visibility)>,
    mut alien_manager: ResMut<AlienManager>,
    resolution: Res<resolution::Resolution>,
    time: Res<Time>,
) {
    for (entity, mut alien, mut transform, mut visibility) in alien_query.iter_mut() {
        transform.translation += Vec3::new(
            alien.velocity.x * time.delta_seconds(),
            alien.velocity.y * time.delta_seconds(),
            0.,
        );

        // Check if the alien has gone out of bounds and bounce it back
        if transform.translation.x.abs() > resolution.screen_dimensions.x * 0.5 - SPACING {
            alien.velocity.x *= -1.;
        }
        if transform.translation.y < -resolution.screen_dimensions.y * 0.5 + SPACING {
            alien.velocity.y *= -1.;
        }

        if transform.translation.y < -resolution.screen_dimensions.y * 0.5 {
            alien.dead = true;
            alien_manager.alive_aliens -= 1;
        }

        if alien.dead {
            commands.entity(entity).insert(Dead {});
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    }
}

fn manage_alien_logic(
    mut commands: Commands,
    mut alien_query: Query<(Entity, &mut Alien, &mut Transform)>,
    mut alien_manager: ResMut<AlienManager>,
) {
    if alien_manager.alive_aliens == 0 {
        alien_manager.reset = true;
        for (entity, mut alien, mut transform) in alien_query.iter_mut() {
            transform.translation = alien.original_position;
            if alien.dead {
                alien.dead = false;
                alien_manager.alive_aliens += 1;
                commands.entity(entity).remove::<Dead>();
            }
        }
    }
}
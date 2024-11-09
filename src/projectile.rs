use bevy::prelude::*;

use crate::resolution;
use crate::alien;
use crate::game::Score;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_projectiles, update_alien_interactions));
    }
}

#[derive(Component)]
pub struct Projectile {
    pub speed: f32,
}

fn update_projectiles(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &Projectile, &mut Transform)>,
    time: Res<Time>,
    resolution: Res<resolution::Resolution>,
) {
    for (entity, projectile, mut transform) in projectile_query.iter_mut() {
        transform.translation.y += projectile.speed * time.delta_seconds();
        if transform.translation.y > resolution.screen_dimensions.y * 0.5 {
            commands.entity(entity).despawn();
        }
    }
}

const BULLET_RADIUS: f32 = 24.0;

fn update_alien_interactions(
    mut alien_query: Query<(&mut alien::Alien, &Transform), Without<alien::Dead>>,
    mut projectile_query: Query<(Entity, &Transform), With<Projectile>>,
    mut commands: Commands,
    mut score: ResMut<Score>,
) {
    let mut despawned_projectiles = Vec::new();
    let mut killed_aliens = Vec::new();

    // First pass: collect all collisions
    for (alien_idx, (alien, alien_transform)) in alien_query.iter().enumerate() {
        if alien.dead {
            continue; // Skip already dead aliens
        }

        let alien_pos = Vec2::new(
            alien_transform.translation.x,
            alien_transform.translation.y,
        );

        for (projectile_entity, projectile_transform) in projectile_query.iter() {
            if despawned_projectiles.contains(&projectile_entity) {
                continue; // Skip already used projectiles
            }

            let projectile_pos = Vec2::new(
                projectile_transform.translation.x,
                projectile_transform.translation.y,
            );

            if Vec2::distance(alien_pos, projectile_pos) < BULLET_RADIUS {
                killed_aliens.push(alien_idx);
                despawned_projectiles.push(projectile_entity);
                break; // One projectile can only kill one alien
            }
        }
    }

    // Second pass: apply the changes
    for alien_idx in killed_aliens {
        if let Some((mut alien, _)) = alien_query.iter_mut().nth(alien_idx) {
            if !alien.dead {  // Double-check the alien isn't already dead
                alien.dead = true;
                score.value += 1;
            }
        }
    }

    // Finally, despawn all used projectiles
    for projectile_entity in despawned_projectiles {
        commands.entity(projectile_entity).despawn();
    }
}
use bevy::prelude::*;

#[derive(Component)]
pub struct Projectile {
    pub speed: f32,
}

pub fn projectile_movement_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Projectile)>,
) {
    for (entity, mut transform, projectile) in query.iter_mut() {
        transform.translation.y += projectile.speed * time.delta_seconds();

        if transform.translation.y > 300.0 {
            commands.entity(entity).despawn();
        }
    }
}

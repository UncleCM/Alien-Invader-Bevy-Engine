use bevy::prelude::*;

#[derive(Component)]
pub struct Alien;

#[derive(Component)]
pub struct AlienPosition(pub Vec2);

#[derive(Component)]
pub struct AlienMovement {
    pub speed: f32,
    pub direction: Vec2,
}

pub fn spawn_aliens(mut commands: Commands, asset_server: Res<AssetServer>) {
    let alien_texture = asset_server.load("alien.png");

    let rows = 5;
    let cols = 10;
    let spacing = 40.0;

    for row in 0..rows {
        for col in 0..cols {
            commands.spawn_bundle(SpriteBundle {
                texture: alien_texture.clone(),
                transform: Transform {
                    translation: Vec3::new(col as f32 * spacing, row as f32 * spacing, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Alien)
            .insert(AlienPosition(Vec2::new(col as f32 * spacing, row as f32 * spacing)))
            .insert(AlienMovement {
                speed: 50.0,
                direction: Vec2::new(1.0, 0.0),
            });
        }
    }
}

pub fn alien_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut AlienMovement)>,
) {
    let screen_width = 800.0;

    for (mut transform, mut movement) in query.iter_mut() {
        transform.translation += movement.direction.extend(0.0) * movement.speed * time.delta_seconds();

        if transform.translation.x > screen_width / 2.0 || transform.translation.x < -screen_width / 2.0 {
            movement.direction.x *= -1.0;
            transform.translation.y -= 20.0;
        }
    }
}

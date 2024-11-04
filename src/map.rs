use bevy::prelude::*;

pub fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_texture = asset_server.load("background.png");

    commands.spawn_bundle(SpriteBundle {
        texture: map_texture,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -1.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

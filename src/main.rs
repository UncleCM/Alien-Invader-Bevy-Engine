use bevy::prelude::*;

mod alien;
mod game;
mod player;
mod projectile;
mod resolution;
mod gyro_hander;
fn main() {
    App::new()
        // Add default Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Space Invaders".into(),
                // Set initial window size
                resolution: (800., 600.).into(),
                // Center the window
                position: WindowPosition::Centered(MonitorSelection::Primary),
                // Prevent window resizing for consistent gameplay
                resizable: false,
                // Set window to be visible immediately
                visible: true,
                ..default()
            }),
            ..default()
        }))
        // Add our game plugin which contains all other plugins
        .add_plugins(game::GamePlugin)
        .add_plugins(gyro_hander::InputPlugin)
        .run();
}
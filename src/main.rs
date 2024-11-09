use bevy::prelude::*;
use bevy::window::{WindowPlugin, Window, WindowPosition, MonitorSelection};

mod alien;
mod game;
mod player;
mod projectile;
mod resolution;
mod gyro_hander;

fn main() {
    App::new()
        // Add default Bevy plugins
        .add_plugins(DefaultPlugins) // This automatically includes the rendering backend configuration
        // Add the custom window settings with a fixed resolution and center it on the primary monitor
        .add_plugins(WindowPlugin {  // Changed from .add_plugin to .add_plugins
            primary_window: Some(Window {
                title: "Space Invaders".into(),
                resolution: (800., 600.).into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                resizable: false, // Make sure the window is not resizable
                visible: true,    // The window is visible immediately
                ..default()
            }),
            ..default()
        })
        // Add your custom game plugin
        .add_plugins(game::GamePlugin)
        // Add the custom input plugin for gyroscope handling or whatever custom logic you have
        .add_plugins(gyro_hander::InputPlugin)
        // Run the application
        .run();
}
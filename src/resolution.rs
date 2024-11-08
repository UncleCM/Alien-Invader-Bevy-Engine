use bevy::prelude::*;

pub struct ResolutionPlugin;

impl Plugin for ResolutionPlugin {
    fn build(&self, app: &mut App) {
        // PreStartup runs before all of our in-game startup functions
        app.add_systems(PreStartup, setup_resolution)
           .add_systems(Startup, setup_background);
    }
}

#[derive(Resource)]
pub struct Resolution {
    // Pixel dimensions of our screen in the form of a 2D vector (width, height)
    pub screen_dimensions: Vec2,
    // The ratio of a pixel in our sprites to one on screen
    pub pixel_ratio: f32,
}

fn setup_resolution(mut commands: Commands, window_query: Query<&Window>) {
    // Query for window information
    let window = window_query.single();

    commands.insert_resource(Resolution {
        screen_dimensions: Vec2::new(window.width(), window.height()),
        pixel_ratio: 2.0,
    });
}

fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    resolution: Res<Resolution>,
) {
    let background_handle = asset_server.load("background.png");
    commands.spawn(SpriteBundle {
        texture: background_handle.clone(),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -1.0), // Ensure the background is behind other elements
            scale: Vec3::new(
                resolution.screen_dimensions.x / 800.0, // Assuming the background image is 100x100 pixels
                resolution.screen_dimensions.y / 600.0,
                1.0,
            ),
            ..Default::default()
        },
        ..Default::default()
    });
}
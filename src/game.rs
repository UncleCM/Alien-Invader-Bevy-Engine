use bevy::prelude::*;

use crate::alien;
use crate::resolution::{self, Resolution};
use crate::player;
use crate::projectile;
pub struct GamePlugin;

#[derive(Resource)]
pub struct Score {
    pub value: usize,
}

#[derive(Component)]
struct ScoreText;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Score { value: 0 })
            .add_plugins(
                (
                    alien::AlienPlugin,
                    resolution::ResolutionPlugin,
                    player::PlayerPlugin,
                    projectile::ProjectilePlugin,
                )
            )
            .add_systems(Startup, (setup_scene, setup_ui))
            .add_systems(Update, update_score_text);
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>, _resolution: Res<Resolution>) {
    commands.spawn(TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..Default::default()
        },
        text: Text::from_section(
            "Score: 0",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        ),
        ..Default::default()
    })
    .insert(ScoreText);
}

fn update_score_text(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        let mut text = query.single_mut();
        text.sections[0].value = format!("Score: {}", score.value);
    }
}
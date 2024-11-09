use bevy::prelude::*;
use crate::alien::{AlienManager, WIDTH, HEIGHT};
use crate::alien;
use crate::resolution::{self, Resolution};
use crate::player;
use crate::projectile;
pub struct GamePlugin;

#[derive(Resource)]
pub struct Score {
    pub value: usize,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Component)]
pub struct GameOverText;

#[derive(Component)]
struct ScoreText;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>()
            .insert_resource(Score { value: 0 })
            .insert_resource(AlienManager {
                reset: false,
                alive_aliens: (WIDTH * HEIGHT) as usize,
            })
            .add_plugins((
                alien::AlienPlugin,
                resolution::ResolutionPlugin,
                player::PlayerPlugin,
                projectile::ProjectilePlugin,
            ))
            .add_systems(Startup, (setup_scene, setup_ui))
            .add_systems(Update, (
                update_score_text,
                check_game_over,
                handle_game_over.run_if(in_state(GameState::GameOver)),
            ));
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
                font: asset_server.load("fonts/Pixel Times Bold.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        ),
        ..Default::default()
    })
    .insert(ScoreText);

    commands.spawn((
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                // Add these new properties for centering
                margin: UiRect {
                    left: Val::Px(-300.0), // Half of the approximate text width
                    top: Val::Px(-50.0),   // Half of the approximate text height
                    ..default()
                },
                ..Default::default()
            },
            text: Text::from_sections([
                TextSection::new(
                    "Game Over!\nPress Space to restart",
                    TextStyle {
                        font: asset_server.load("fonts/Pixel Times Bold.ttf"),
                        font_size: 50.0,
                        color: Color::rgb(1.0, 0.0, 0.0),
                    },
                )
            ]).with_justify(JustifyText::Center),
            visibility: Visibility::Hidden,
            ..Default::default()
        },
        GameOverText,
    ));
}

fn update_score_text(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        let mut text = query.single_mut();
        text.sections[0].value = format!("Score: {}", score.value);
    }
}

fn check_game_over(
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<&Transform, With<player::Player>>,
    alien_query: Query<&Transform, With<alien::Alien>>,
    current_state: Res<State<GameState>>,
) {
    if current_state.get() != &GameState::Playing {
        return;
    }

    if let Ok(player_transform) = player_query.get_single() {
        let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

        // Check collision with aliens
        for alien_transform in alien_query.iter() {
            let alien_pos = Vec2::new(alien_transform.translation.x, alien_transform.translation.y);
            if Vec2::distance(player_pos, alien_pos) < 32.0 {
                next_state.set(GameState::GameOver);
                return;
            }
        }
    }
}

fn handle_game_over(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_over_text: Query<&mut Visibility, With<GameOverText>>,
    mut score: ResMut<Score>,
    mut alien_manager: ResMut<AlienManager>,
    aliens: Query<Entity, With<alien::Alien>>,
    projectiles: Query<Entity, Or<(With<projectile::Projectile>, With<alien::AlienProjectile>)>>,
    mut player: Query<&mut Transform, With<player::Player>>,
) {
    // Show game over text
    if let Ok(mut visibility) = game_over_text.get_single_mut() {
        *visibility = Visibility::Visible;
    }

    // Reset game when space is pressed
    if keyboard.just_pressed(KeyCode::Space) {
        // Hide game over text
        if let Ok(mut visibility) = game_over_text.get_single_mut() {
            *visibility = Visibility::Hidden;
        }

        // Reset score
        score.value = 0;

        // Reset alien manager
        alien_manager.reset = true;
        alien_manager.alive_aliens = (WIDTH * HEIGHT) as usize;

        // Despawn all aliens
        for entity in aliens.iter() {
            commands.entity(entity).despawn();
        }

        // Despawn all projectiles
        for entity in projectiles.iter() {
            commands.entity(entity).despawn();
        }

        // Reset player position
        if let Ok(mut transform) = player.get_single_mut() {
            transform.translation.x = 0.0;
        }

        // Switch back to playing state
        next_state.set(GameState::Playing);
    }
}
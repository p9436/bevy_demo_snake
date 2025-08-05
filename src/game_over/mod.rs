use bevy::prelude::*;

use crate::{GameState, Score};

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_game_over);
        app.add_systems(OnEnter(GameState::GameOver), show_game_over);
        app.add_systems(Update, handle_inputs.run_if(in_state(GameState::GameOver)));
        app.add_systems(OnExit(GameState::GameOver), hide_game_over);
    }
}

#[derive(Component)]
struct GameOverText;

fn handle_inputs(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.pressed(KeyCode::KeyR) {
        game_state.set(GameState::InGame);
    }
}

fn init_game_over(mut commands: Commands) {
    commands.spawn((
        Text::new(format!("GAME OVER\nScore: {}\nPress R to restart", 0)),
        TextLayout::new_with_justify(JustifyText::Center),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        GameOverText,
    ));
}

fn show_game_over(
    score: Res<Score>,
    mut query: Query<(&mut Visibility, &mut Text), With<GameOverText>>,
) {
    if let Ok((mut visibility, mut text)) = query.single_mut() {
        *visibility = Visibility::Visible;
        text.0 = format!("GAME OVER\nScore: {}\nPress R to restart", score.0);
    }
}

fn hide_game_over(mut game_over_query: Query<&mut Visibility, With<GameOverText>>) {
    if let Ok(mut visibility) = game_over_query.single_mut() {
        *visibility = Visibility::Hidden;
    }
}

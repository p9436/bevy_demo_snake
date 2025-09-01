use bevy::prelude::*;

use crate::GameState;

pub struct GamePausePlugin;

impl Plugin for GamePausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_pause_message);
        app.add_systems(OnEnter(GameState::Paused), show_pause);
        app.add_systems(OnExit(GameState::Paused), hide_pause);
        app.add_systems(
            Update,
            handle_inputs_in_game.run_if(in_state(GameState::InGame)),
        );
        app.add_systems(
            Update,
            handle_inputs_on_pause.run_if(in_state(GameState::Paused)),
        );
    }
}

#[derive(Component)]
struct PauseText;

fn handle_inputs_in_game(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_state.set(GameState::Paused);
    }
}

fn handle_inputs_on_pause(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_state.set(GameState::InGame);
    }
}

fn init_pause_message(mut commands: Commands) {
    commands.spawn((
        Text::new("Paused"),
        TextLayout::new_with_justify(JustifyText::Center),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        PauseText,
    ));
}

fn show_pause(mut query: Query<&mut Visibility, With<PauseText>>) {
    if let Ok(mut visibility) = query.single_mut() {
        *visibility = Visibility::Visible;
    }
}

fn hide_pause(mut query: Query<&mut Visibility, With<PauseText>>) {
    if let Ok(mut visibility) = query.single_mut() {
        *visibility = Visibility::Hidden;
    }
}

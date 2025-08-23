use bevy::prelude::*;
use rand::Rng;

use crate::snake::{Ate, Head};

const FIELD_WIDTH: u8 = 10;
const FIELD_HEIGHT: u8 = 10;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    AssetsLoading,
    InGame,
    GameOver,
}

mod assets_loader;
mod game_over;
mod snake;

#[derive(Debug, Component, Clone, Copy)]
pub struct Position {
    x: i8,
    y: i8,
}

#[derive(Component)]
struct BorderSegment;

#[derive(Resource, Default)]
struct Score(usize);

#[derive(Component)]
struct Food;

#[derive(Component)]
struct FpsText;

fn setup(mut commands: Commands) {
    // Camera with 4x pixel scaling
    commands.spawn((Camera2d, Transform::from_scale(Vec3::splat(0.25))));

    // FPS Text
    commands.spawn((
        Text::new("FPS: 0"),
        TextLayout::new_with_justify(JustifyText::Left),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            ..default()
        },
        FpsText,
    ));

    spawn_border(&mut commands);

    // Food
    let position = Position { x: 3, y: 3 };
    let screen_position = grid_to_screen_transform(&position);
    commands.spawn((
        Food,
        Sprite {
            color: Color::srgb(0.8, 0.3, 0.3),
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        position,
        screen_position,
    ));
}

fn grid_to_screen_position(position: &Position) -> Vec3 {
    grid_to_screen_transform(position).translation
}

fn grid_to_screen_transform(position: &Position) -> Transform {
    Transform::from_xyz(position.x as f32 * 8.0, position.y as f32 * 8.0, 0.0)
}

fn update_fps(time: Res<Time>, mut fps_query: Query<&mut Text, With<FpsText>>) {
    if let Ok(mut fps_text) = fps_query.single_mut() {
        let fps = 1.0 / time.delta_secs();
        fps_text.0 = format!("FPS: {:.0}", fps);
    }
}

fn check_border_collision(
    mut head_query: Query<&Position, With<Head>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Ok(head_pos) = head_query.single_mut() {
        if head_pos.x < 0
            || head_pos.x >= FIELD_WIDTH as i8
            || head_pos.y < 0
            || head_pos.y >= FIELD_HEIGHT as i8
        {
            println!("Game Over");
            next_state.set(GameState::GameOver);
        }
    }
}

fn check_food_collision(
    mut food_query: Query<(&mut Position, &mut Transform), With<Food>>,
    mut head_query: Query<(&Position, &mut Ate), (With<Head>, Without<Food>)>,
    mut score: ResMut<Score>,
) {
    if let Ok((mut food_pos, mut food_transform)) = food_query.single_mut() {
        if let Ok((head_pos, mut snake_ate)) = head_query.single_mut() {
            if head_pos.x == food_pos.x && head_pos.y == food_pos.y {
                food_pos.x = rand::rng().random_range(0..FIELD_WIDTH as i8);
                food_pos.y = rand::rng().random_range(0..FIELD_HEIGHT as i8);

                food_transform.translation = grid_to_screen_position(&food_pos);

                snake_ate.0 = true;

                score.0 += 1;
                println!("Score: {}", score.0);
            }
        }
    }
}

fn spawn_border(commands: &mut Commands) {
    let mut border = Vec::new();
    for x in 0..FIELD_WIDTH + 2 {
        border.push((x, 0));
        border.push((x, FIELD_HEIGHT + 1));
    }
    for y in 0..FIELD_HEIGHT {
        border.push((0, y + 1));
        border.push((FIELD_WIDTH + 1, y + 1));
    }
    border.into_iter().for_each(|(x, y)| {
        let pos = Position {
            x: x as i8 - 1,
            y: y as i8 - 1,
        };
        let screen_pos = grid_to_screen_transform(&pos);
        commands.spawn((
            BorderSegment,
            pos,
            screen_pos,
            Sprite {
                color: Color::srgb(0.6, 0.6, 0.6),
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..default()
            },
        ));
    });
}

fn reset_score(mut score: ResMut<Score>) {
    score.0 = 0;
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Snake".to_string(),
                        resolution: (800.0, 600.0).into(),

                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(assets_loader::AssetsLoaderPlugin)
        .add_plugins(snake::SnakePlugin)
        .add_plugins(game_over::GameOverPlugin)
        .init_state::<GameState>()
        .init_resource::<Score>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::InGame), reset_score)
        .add_systems(
            Update,
            (check_border_collision, check_food_collision)
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(Update, update_fps)
        .run();
}

use bevy::prelude::*;
use rand::Rng;

const TIMER_TURN_DELAY: f32 = 0.8;
const FIELD_WIDTH: u8 = 10;
const FIELD_HEIGHT: u8 = 10;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    InGame,
    GameOver,
}

mod game;
mod game_over;

#[derive(Debug, Component, Clone, Copy)]
struct Position {
    x: i8,
    y: i8,
}

#[derive(Component)]
struct BorderSegment;

#[derive(Component)]
struct Head;

#[derive(Component)]
struct Ate(bool);

#[derive(Resource)]
struct Timer(f32);

#[derive(Resource, Default)]
struct Score(usize);

#[derive(Copy, Clone, PartialEq)]
enum Dir {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Component)]
struct Direction(Dir);

#[derive(Component)]
struct LastDirection(Dir);

#[derive(Component)]
struct BodySegment;

#[derive(Component)]
struct NextSegment(Entity);

#[derive(Component)]
struct Food;

#[derive(Component)]
struct FpsText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(game_over::GameOverPlugin)
        .init_state::<GameState>()
        .init_resource::<Score>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::InGame), (reset_game, setup_game))
        .add_systems(
            Update,
            (
                player_input,
                update_timer,
                move_snake,
                check_border_collision,
                check_self_collision,
                check_food_collision,
                reset_timer,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(Update, update_fps)
        .run();
}

fn setup(mut commands: Commands) {
    // Timer
    commands.insert_resource(Timer(TIMER_TURN_DELAY));

    // // Score
    // commands.insert_resource(Score(0));

    // Camera
    commands.spawn(Camera2d);

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
    let screen_position = grid_to_screen_position(&position);
    commands.spawn((
        Food,
        Sprite {
            color: Color::srgb(0.8, 0.3, 0.3),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        position,
        screen_position,
    ));
}

fn grid_to_screen_position(position: &Position) -> Transform {
    Transform::from_xyz(position.x as f32 * 32.0, position.y as f32 * 32.0, 0.0)
}

fn update_fps(time: Res<Time>, mut fps_query: Query<&mut Text, With<FpsText>>) {
    if let Ok(mut fps_text) = fps_query.single_mut() {
        let fps = 1.0 / time.delta_secs();
        fps_text.0 = format!("FPS: {:.0}", fps);
    }
}

fn update_timer(time: Res<Time>, mut timer: ResMut<Timer>) {
    timer.0 -= time.delta_secs();
}

fn reset_timer(mut timer: ResMut<Timer>) {
    if timer.0 < 0.0 {
        timer.0 = TIMER_TURN_DELAY;
    }
}

fn player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut head_query: Query<(&mut Direction, &LastDirection), With<Head>>,
) {
    if let Ok(mut head) = head_query.single_mut() {
        let last_direction = head.1.0;

        if keyboard_input.pressed(KeyCode::KeyA) && last_direction != Dir::Right {
            head.0.0 = Dir::Left;
        } else if keyboard_input.pressed(KeyCode::KeyD) && last_direction != Dir::Left {
            head.0.0 = Dir::Right;
        } else if keyboard_input.pressed(KeyCode::KeyW) && last_direction != Dir::Down {
            head.0.0 = Dir::Up;
        } else if keyboard_input.pressed(KeyCode::KeyS) && last_direction != Dir::Up {
            head.0.0 = Dir::Down;
        }
    }
}

fn move_snake(
    mut commands: Commands,
    timer: Res<Timer>,
    mut head_query: Query<
        (
            &mut Position,
            &mut Transform,
            &mut LastDirection,
            &mut Ate,
            &Direction,
            &NextSegment,
        ),
        With<Head>,
    >,
    mut body_query: Query<
        (Entity, &mut Position, &mut Transform, Option<&NextSegment>),
        (With<BodySegment>, Without<Head>),
    >,
) {
    if timer.0 > 0.0 {
        return;
    }

    if let Ok((
        mut head_position,
        mut head_transform,
        mut head_last_direction,
        mut head_ate,
        head_direction,
        head_next_segment,
    )) = head_query.single_mut()
    {
        let prev_head_pos = *head_position;

        match head_direction.0 {
            Dir::Left => head_position.x -= 1,
            Dir::Right => head_position.x += 1,
            Dir::Up => head_position.y += 1,
            Dir::Down => head_position.y -= 1,
        }

        head_last_direction.0 = head_direction.0;

        head_transform.translation = grid_to_screen_position(&head_position).translation;

        let mut current_segment_id = head_next_segment.0;
        let mut prev_pos = prev_head_pos;
        let mut last_segment_entity: Option<Entity> = None;

        loop {
            if let Ok((entity, mut segment_pos, mut segment_transform, next_segment)) =
                body_query.get_mut(current_segment_id)
            {
                let old_segment_pos = *segment_pos;
                *segment_pos = prev_pos;
                segment_transform.translation = grid_to_screen_position(&segment_pos).translation;
                prev_pos = old_segment_pos;

                if let Some(next) = next_segment {
                    current_segment_id = next.0;
                } else {
                    // We've reached the end of the snake.
                    last_segment_entity = Some(entity);
                    break;
                }
            } else {
                break;
            }
        }

        if head_ate.0 {
            if let Some(last_entity) = last_segment_entity {
                head_ate.0 = false;

                let new_segment_pos = prev_pos;
                let new_segment_entity = spawn_body_segment(&mut commands, &new_segment_pos);

                commands
                    .entity(last_entity)
                    .insert(NextSegment(new_segment_entity));
            }
        }
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

fn check_self_collision(
    head_query: Query<&Position, With<Head>>,
    body_query: Query<&Position, (With<BodySegment>, Without<Head>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Ok(head_pos) = head_query.single() {
        for body_pos in body_query.iter() {
            if head_pos.x == body_pos.x && head_pos.y == body_pos.y {
                println!("Game Over");
                next_state.set(GameState::GameOver);
                break;
            }
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

                food_transform.translation = grid_to_screen_position(&food_pos).translation;

                snake_ate.0 = true;

                score.0 += 1;
                println!("Score: {}", score.0);
            }
        }
    }
}

fn spawn_body_segment(commands: &mut Commands, position: &Position) -> Entity {
    let new_screen_position = grid_to_screen_position(position);
    let new_segment_entity = commands
        .spawn((
            BodySegment,
            *position,
            new_screen_position,
            Sprite {
                color: Color::srgb(0.3, 0.8, 0.3),
                custom_size: Some(Vec2::new(31.0, 31.0)),
                ..default()
            },
        ))
        .id();
    new_segment_entity
}

fn spawn_head(commands: &mut Commands, position: &Position, initial_body_segment: Entity) {
    let screen_position = grid_to_screen_position(position);
    commands.spawn((
        Head,
        Sprite {
            color: Color::srgb(0.3, 0.8, 0.3),
            custom_size: Some(Vec2::new(31.0, 31.0)),
            ..default()
        },
        *position,
        screen_position,
        Direction(Dir::Right),
        LastDirection(Dir::Right),
        NextSegment(initial_body_segment),
        Ate(false),
    ));
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
        let screen_pos = grid_to_screen_position(&pos);
        commands.spawn((
            BorderSegment,
            pos,
            screen_pos,
            Sprite {
                color: Color::srgb(0.6, 0.6, 0.6),
                custom_size: Some(Vec2::new(31.0, 31.0)),
                ..default()
            },
        ));
    });
}

fn reset_game(
    mut commands: Commands,
    head_query: Query<Entity, With<Head>>,
    body_query: Query<Entity, With<BodySegment>>,
    mut score: ResMut<Score>,
) {
    score.0 = 0;

    for entity in head_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in body_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn setup_game(mut commands: Commands) {
    // BodySegment
    let position = Position { x: 4, y: 5 };
    let initial_body_segment = spawn_body_segment(&mut commands, &position);

    // Head
    let position = Position { x: 5, y: 5 };
    spawn_head(&mut commands, &position, initial_body_segment);
}

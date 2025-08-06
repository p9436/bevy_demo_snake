use bevy::prelude::*;
use rand::Rng;

use crate::{GameState, Position, grid_to_screen_position};

const TIMER_TURN_DELAY: f32 = 0.8;

pub struct SnakePlugin;

#[derive(Component)]
pub struct Head;

#[derive(Component)]
pub struct BodySegment;

#[derive(Component)]
struct NextSegment(Entity);

#[derive(Component)]
pub struct Ate(bool);

#[derive(Resource)]
struct Timer(f32);

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

//

impl Plugin for SnakePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);

        app.add_systems(OnEnter(GameState::InGame), (despawn_snake, init_snake));

        app.add_systems(
            Update,
            (
                handle_inputs,
                update_timer,
                movements,
                check_self_collision,
                reset_timer,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        );
    }
}

fn startup(mut commands: Commands) {
    // Timer
    commands.insert_resource(Timer(TIMER_TURN_DELAY));
}

fn update_timer(time: Res<Time>, mut timer: ResMut<Timer>) {
    timer.0 -= time.delta_secs();
}

fn reset_timer(mut timer: ResMut<Timer>) {
    if timer.0 < 0.0 {
        timer.0 = TIMER_TURN_DELAY;
    }
}

fn handle_inputs(
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

fn movements(
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

fn despawn_snake(
    mut commands: Commands,
    head_query: Query<Entity, With<Head>>,
    body_query: Query<Entity, With<BodySegment>>,
) {
    for entity in head_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in body_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn init_snake(mut commands: Commands) {
    // BodySegment
    let position = Position { x: 4, y: 5 };
    let initial_body_segment = spawn_body_segment(&mut commands, &position);

    // Head
    let position = Position { x: 5, y: 5 };
    spawn_head(&mut commands, &position, initial_body_segment);
}

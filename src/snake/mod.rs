use bevy::prelude::*;

use crate::{
    GameState, Position, assets_loader::GameAssets, grid_to_screen_position,
    grid_to_screen_transform,
};

const TIMER_TURN_DELAY: f32 = 0.8;

pub struct SnakePlugin;

#[derive(Component)]
pub struct Head;

#[derive(Component)]
pub struct BodySegment;

#[derive(Component)]
struct NextSegment(Entity);

#[derive(Component)]
pub struct Ate(pub bool);

#[derive(Resource)]
struct Timer(f32);

#[derive(Copy, Clone, PartialEq, Debug)]
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

// Enum to represent different types of body segments
#[derive(Debug, Clone, Copy, PartialEq)]
enum SegmentType {
    // Straight segments
    Horizontal, // Left-Right or Right-Left
    Vertical,   // Up-Down or Down-Up

    // Corner segments (turn from one direction to another)
    CornerRightUp,   // From Right to Up
    CornerDownRight, // From Down to Right
    CornerLeftDown,  // From Left to Down
    CornerUpLeft,    // From Up to Left

    // Tail segment (last segment of the snake)
    TailUp,
    TailRight,
    TailDown,
    TailLeft,

    // Fallback
    None,
}

impl SegmentType {
    // Map segment types to sprite atlas indices
    fn to_atlas_index(self) -> usize {
        match self {
            SegmentType::Horizontal => 4,      // Horizontal straight segment
            SegmentType::Vertical => 5,        // Vertical straight segment
            SegmentType::CornerDownRight => 8, //
            SegmentType::CornerLeftDown => 9,  //
            SegmentType::CornerUpLeft => 13,   //
            SegmentType::CornerRightUp => 12,  //
            SegmentType::TailRight => 11,
            SegmentType::TailDown => 14,
            SegmentType::TailLeft => 15,
            SegmentType::TailUp => 10,
            SegmentType::None => 7,
        }
    }
}

// Helper function to get direction between two positions
fn get_direction_between_positions(from: &Position, to: &Position) -> Option<Dir> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;

    // println!("dx, dy: {:?},{:?}", dx, dy);

    match (dx, dy) {
        (1, 0) => Some(Dir::Right),
        (-1, 0) => Some(Dir::Left),
        (0, 1) => Some(Dir::Up),
        (0, -1) => Some(Dir::Down),
        _ => None, // Not adjacent or diagonal
    }
}

// Function to determine what type of segment this should be
fn determine_segment_type(
    prev_pos: &Position,
    current_pos: &Position,
    next_pos: &Position,
) -> SegmentType {
    let direction_from_prev = get_direction_between_positions(prev_pos, current_pos);
    let direction_to_next = get_direction_between_positions(current_pos, next_pos);

    // Convert to CONNECTION directions (what directions the current segment connects to)
    let incoming_connection = match direction_from_prev {
        Some(Dir::Up) => Some(Dir::Down), // If prev is above, connection comes from Up
        Some(Dir::Down) => Some(Dir::Up), // If prev is below, connection comes from Down
        Some(Dir::Left) => Some(Dir::Right), // If prev is left, connection comes from Left
        Some(Dir::Right) => Some(Dir::Left), // If prev is right, connection comes from Right
        None => None,
    };

    let outgoing_connection = direction_to_next; // This one is correct as-is

    // println!(
    // "prev: {:?} -> curr: {:?} -> next: {:?}",
    // prev_pos, current_pos, next_pos
    // );
    // println!(
    // "dir_from_prev: {:?}, dir_to_next: {:?}",
    // direction_from_prev, direction_to_next
    // );

    match (incoming_connection, outgoing_connection) {
        (Some(Dir::Left), Some(Dir::Left)) | (Some(Dir::Right), Some(Dir::Right)) => {
            SegmentType::Horizontal
        }
        (Some(Dir::Up), Some(Dir::Up)) | (Some(Dir::Down), Some(Dir::Down)) => {
            SegmentType::Vertical
        }

        // Straight segments - OPPOSITE DIRECTIONS (less common, but possible)
        (Some(Dir::Left), Some(Dir::Right)) | (Some(Dir::Right), Some(Dir::Left)) => {
            SegmentType::Horizontal
        }
        (Some(Dir::Up), Some(Dir::Down)) | (Some(Dir::Down), Some(Dir::Up)) => {
            SegmentType::Vertical
        }

        //
        (Some(Dir::Down), Some(Dir::Right)) => SegmentType::CornerDownRight,
        (Some(Dir::Left), Some(Dir::Down)) => SegmentType::CornerLeftDown,
        (Some(Dir::Up), Some(Dir::Left)) => SegmentType::CornerUpLeft,
        (Some(Dir::Right), Some(Dir::Up)) => SegmentType::CornerRightUp,
        //
        (Some(Dir::Right), Some(Dir::Down)) => SegmentType::CornerDownRight,
        (Some(Dir::Down), Some(Dir::Left)) => SegmentType::CornerLeftDown,
        (Some(Dir::Left), Some(Dir::Up)) => SegmentType::CornerUpLeft,
        (Some(Dir::Up), Some(Dir::Right)) => SegmentType::CornerRightUp,

        // Fallback to straight segments if we can't determine corner
        _ => {
            // println!(
            // "Fallback case reached for: {:?} -> {:?}",
            // direction_from_prev, direction_to_next
            // );
            SegmentType::None
        }
    }
}

fn determine_tail_type(prev_pos: &Position, tail_pos: &Position) -> SegmentType {
    let direction_to_next = get_direction_between_positions(prev_pos, tail_pos);

    // println!("prev: {:?} -> tail: {:?}", prev_pos, tail_pos);

    match direction_to_next {
        Some(Dir::Left) => SegmentType::TailLeft,
        Some(Dir::Right) => SegmentType::TailRight,
        Some(Dir::Up) => SegmentType::TailUp,
        Some(Dir::Down) => SegmentType::TailDown,

        // Fallback to straight segments if we can't determine corner
        _ => {
            // println!("Fallback case reached for: {:?} ", direction_to_next);
            SegmentType::None
        }
    }
}

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

fn spawn_head(
    commands: &mut Commands,
    position: &Position,
    initial_body_segment: Entity,
    game_assets: Res<GameAssets>,
) {
    let screen_position = grid_to_screen_transform(position);
    commands.spawn((
        Head,
        Sprite {
            image: game_assets.texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: game_assets.texture_atlas_layout.clone(),
                index: 1,
                ..default()
            }),
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

fn init_snake(mut commands: Commands, game_assets: Res<GameAssets>) {
    // BodySegment
    let position = Position { x: 0, y: 0 };
    let initial_body_segment = spawn_body_segment(&mut commands, &position, &game_assets);

    // Head
    let position = Position { x: 1, y: 0 };
    spawn_head(&mut commands, &position, initial_body_segment, game_assets);
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
            Entity,
            &mut Position,
            &mut Sprite,
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
    game_assets: Res<GameAssets>,
) {
    if timer.0 > 0.0 {
        return;
    }

    if let Ok((
        head_entity,
        mut head_pos,
        mut sprite,
        mut head_transform,
        mut head_last_direction,
        mut snake_ate,
        head_direction,
        head_next_segment,
    )) = head_query.single_mut()
    {
        let prev_head_pos = *head_pos;

        // Update head sprite and position
        match head_direction.0 {
            Dir::Left => {
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.index = 3;
                }
                head_pos.x -= 1;
            }
            Dir::Right => {
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.index = 1;
                }
                head_pos.x += 1;
            }
            Dir::Up => {
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.index = 0;
                }
                head_pos.y += 1;
            }
            Dir::Down => {
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.index = 2;
                }
                head_pos.y -= 1;
            }
        }

        head_last_direction.0 = head_direction.0;

        head_transform.translation = grid_to_screen_position(&head_pos);

        let mut ordered_segments = vec![(head_entity, *head_pos)];

        let mut current_segment_id = head_next_segment.0;
        let mut prev_pos = prev_head_pos;
        let mut last_segment_entity: Option<Entity> = None;

        loop {
            if let Ok((entity, mut segment_pos, mut segment_transform, next_segment)) =
                body_query.get_mut(current_segment_id)
            {
                let old_segment_pos = *segment_pos;
                *segment_pos = prev_pos;
                segment_transform.translation = grid_to_screen_position(&segment_pos);
                prev_pos = old_segment_pos;
                ordered_segments.push((current_segment_id, *segment_pos));

                if let Some(next) = next_segment {
                    current_segment_id = next.0;
                } else {
                    last_segment_entity = Some(entity);
                    break;
                }
            } else {
                break;
            }
        }

        if snake_ate.0 {
            if let Some(last_entity) = last_segment_entity {
                snake_ate.0 = false;

                let new_segment_pos = prev_pos;
                let new_segment_entity =
                    spawn_body_segment(&mut commands, &new_segment_pos, &game_assets);

                commands
                    .entity(last_entity)
                    .insert(NextSegment(new_segment_entity));

                ordered_segments.push((new_segment_entity, new_segment_pos));
            }
        }

        // println!("------------");
        // println!("{:?}", ordered_segments);

        let len = ordered_segments.len();
        if len >= 3 {
            for idx in 1..len - 1 {
                let prev = ordered_segments[idx - 1];
                let next = ordered_segments[idx + 1];
                let curr = ordered_segments[idx];

                let segment_type = determine_segment_type(&prev.1, &curr.1, &next.1);
                let atlas_index = segment_type.to_atlas_index();

                let mut entity_commands = commands.entity(curr.0);
                entity_commands.queue(move |mut entity: EntityWorldMut| {
                    // Check if the component exists on the entity.
                    if let Some(mut sprite) = entity.get_mut::<Sprite>() {
                        if let Some(ref mut atlas) = sprite.texture_atlas {
                            atlas.index = atlas_index;
                        }
                    }
                });
            }
        }

        if len >= 2 {
            let prev = ordered_segments[len - 2];
            let tail = ordered_segments[len - 1];

            let segment_type = determine_tail_type(&prev.1, &tail.1);
            let atlas_index = segment_type.to_atlas_index();

            let mut entity_commands = commands.entity(tail.0);
            entity_commands.queue(move |mut entity: EntityWorldMut| {
                // Check if the component exists on the entity.
                if let Some(mut sprite) = entity.get_mut::<Sprite>() {
                    if let Some(ref mut atlas) = sprite.texture_atlas {
                        atlas.index = atlas_index;
                    }
                }
            });
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

fn spawn_body_segment(
    commands: &mut Commands,
    position: &Position,
    game_assets: &Res<GameAssets>,
) -> Entity {
    let new_screen_position = grid_to_screen_transform(position);
    let new_segment_entity = commands
        .spawn((
            BodySegment,
            *position,
            new_screen_position,
            Sprite {
                image: game_assets.texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: game_assets.texture_atlas_layout.clone(),
                    index: 15, // Tail segment
                    ..default()
                }),
                ..default()
            },
        ))
        .id();
    new_segment_entity
}

use bevy::text::JustifyText;
use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
}; // Correct import for SpatialBundle

use rand::Rng;

use crate::{
    assets_loader::GameAssets,
    snake::{Ate, Head},
};

const FIELD_FROM: (i8, i8) = (-5, -5);
const FIELD_TO: (i8, i8) = (6, 6);
const TILE_SIZE: f32 = 8.0;

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

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct Tilemap;

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

    let world_pos = Vec3::new(
        FIELD_FROM.0 as f32 * TILE_SIZE,
        FIELD_TO.1 as f32 * TILE_SIZE + TILE_SIZE * 2.0,
        1.0, // A z-value to ensure the text is rendered on top of other sprites.
    );

    // Score Text (positioned in world space using Text2d)
    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont {
            font_size: 4.0, // Larger font size to compensate for camera scaling
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_translation(world_pos),
        ScoreText,
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
        if head_pos.x <= FIELD_FROM.0 - 1
            || head_pos.x >= FIELD_TO.0 + 1
            || head_pos.y <= FIELD_FROM.1 - 1
            || head_pos.y >= FIELD_TO.1 + 1
        {
            println!("Head: {:?}", head_pos);
            println!("Game Over");
            next_state.set(GameState::GameOver);
        }
    }
}

fn check_food_collision(
    mut food_query: Query<(&mut Position, &mut Transform), With<Food>>,
    mut head_query: Query<(&Position, &mut Ate), (With<Head>, Without<Food>)>,
    mut score_text_query: Query<&mut Text2d, With<ScoreText>>,
    mut score: ResMut<Score>,
) {
    if let Ok((mut food_pos, mut food_transform)) = food_query.single_mut() {
        if let Ok((head_pos, mut snake_ate)) = head_query.single_mut() {
            if head_pos.x == food_pos.x && head_pos.y == food_pos.y {
                food_pos.x = rand::rng().random_range(FIELD_FROM.0..FIELD_TO.0);
                food_pos.y = rand::rng().random_range(FIELD_FROM.1..FIELD_TO.1);

                food_transform.translation = grid_to_screen_position(&food_pos);

                snake_ate.0 = true;

                score.0 += 1;
                println!("Score: {}", score.0);

                if let Ok(mut score_text) = score_text_query.single_mut() {
                    score_text.0 = format!("Score: {}", score.0);
                }
            }
        }
    }
}

fn spawn_food(mut commands: Commands, game_assets: Res<GameAssets>) {
    // Food
    let position = Position { x: 3, y: 3 };
    let screen_position = grid_to_screen_transform(&position);
    commands.spawn((
        Food,
        Sprite {
            image: game_assets.texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: game_assets.texture_atlas_layout.clone(),
                index: 19,
                ..default()
            }),
            ..default()
        },
        position,
        screen_position,
    ));
}

fn spawn_borders(mut commands: Commands, game_assets: Res<GameAssets>) {
    let mut border = Vec::new();

    // Horizontal borders (top and bottom)
    for x in (FIELD_FROM.0 - 1)..=(FIELD_TO.0 + 1) {
        border.push((x, FIELD_TO.1 + 1)); // Top border
        border.push((x, FIELD_FROM.1 - 1)); // Bottom border
    }

    // Vertical borders (left and right)
    for y in (FIELD_FROM.1 - 1)..=(FIELD_TO.1 + 1) {
        border.push((FIELD_FROM.0 - 1, y)); // Left border
        border.push((FIELD_TO.0 + 1, y)); // Right border
    }

    border.into_iter().for_each(|(x, y)| {
        let pos = Position { x, y };
        let screen_pos = grid_to_screen_transform(&pos);
        commands.spawn((
            BorderSegment,
            pos,
            screen_pos,
            Sprite {
                image: game_assets.texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: game_assets.texture_atlas_layout.clone(),
                    index: 17,
                    ..default()
                }),
                ..default()
            },
        ));
    });
}

fn reset_score(mut score: ResMut<Score>) {
    score.0 = 0;
}

fn setup_tilemap_simple(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_assets: Res<GameAssets>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
) {
    // Перевіряємо чи завантажився atlas layout
    let Some(atlas_layout) = texture_atlas_layouts.get(&game_assets.texture_atlas_layout) else {
        println!("TextureAtlasLayout not loaded yet");
        return;
    };

    create_tilemap_mesh(
        &mut commands,
        &mut meshes,
        &mut materials,
        atlas_layout,
        &game_assets.texture,
    );
}

fn create_tilemap_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    atlas_layout: &TextureAtlasLayout,
    texture_handle: &Handle<Image>,
) {
    let map_width = FIELD_TO.0 - FIELD_FROM.0 + 1;
    let map_height = FIELD_TO.1 - FIELD_FROM.1 + 1;
    let tile_size = 8.0;

    let mut vertices = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for y in 0..map_height {
        for x in 0..map_width {
            let vertex_index = vertices.len() as u32;

            let x_pos = x as f32 * tile_size;
            let y_pos = y as f32 * tile_size;

            vertices.extend([
                [x_pos, y_pos, 0.0],
                [x_pos + tile_size, y_pos, 0.0],
                [x_pos + tile_size, y_pos + tile_size, 0.0],
                [x_pos, y_pos + tile_size, 0.0],
            ]);

            // Використовуємо тайл з індексом 16 (або можете зробити рандомний)
            let tile_index = 16;

            // Перевіряємо чи існує тайл з таким індексом
            if tile_index < atlas_layout.textures.len() {
                let tile_rect = atlas_layout.textures[tile_index];

                let atlas_size = atlas_layout.size.as_vec2();
                let u_min = tile_rect.min.x as f32 / atlas_size.x;
                let u_max = tile_rect.max.x as f32 / atlas_size.x;
                let v_min = tile_rect.min.y as f32 / atlas_size.y;
                let v_max = tile_rect.max.y as f32 / atlas_size.y;

                uvs.extend([
                    [u_min, v_max],
                    [u_max, v_max],
                    [u_max, v_min],
                    [u_min, v_min],
                ]);
            } else {
                println!("Sprite index out of range");
            }

            indices.extend([
                vertex_index,
                vertex_index + 1,
                vertex_index + 2,
                vertex_index,
                vertex_index + 2,
                vertex_index + 3,
            ]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(ColorMaterial::from(texture_handle.clone()));

    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform::from_translation(Vec3::new(
            -(map_width as f32 * tile_size) / 2.0 + tile_size / 2.0,
            -(map_height as f32 * tile_size) / 2.0 + tile_size / 2.0,
            -10.0, // Далеко позаду всіх інших об'єктів
        )),
        Tilemap,
    ));

    println!("Tilemap created successfully!");
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
        .add_systems(
            PostStartup,
            (setup_tilemap_simple, spawn_borders, spawn_food).chain(),
        )
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

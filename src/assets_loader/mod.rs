use bevy::asset::LoadState;
use bevy::prelude::*;

use crate::GameState;
pub struct AssetsLoaderPlugin;

impl Plugin for AssetsLoaderPlugin {
    fn build(&self, app: &mut App) {
        // Додаємо систему завантаження ресурсів під час запуску
        app.add_systems(Startup, load_game_assets);
    }
}

#[derive(Resource)]
pub struct GameAssets {
    pub snake_texture_atlas_layout: Handle<TextureAtlasLayout>,
    pub snake_texture: Handle<Image>,
}

fn load_game_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Завантажуємо текстуру спрайтового аркуша.
    let texture = asset_server.load("snake.png");

    // Визначаємо макет спрайтового аркуша: клітинки 8x8 пікселі, 4 стовпців, 4 рядки.
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(8), 4, 4, None, None);
    // Додаємо макет до сервера ресурсів та отримуємо його Handle.
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // Вставляємо ресурс GameAssets у світ, щоб інші системи могли до нього отримати доступ.
    commands.insert_resource(GameAssets {
        snake_texture_atlas_layout: texture_atlas_layout,
        snake_texture: texture,
    });

    next_state.set(GameState::InGame);
}

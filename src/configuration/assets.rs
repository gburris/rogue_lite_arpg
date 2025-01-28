use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::labels::states::AppState;

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .continue_to_state(AppState::SpawnPlayer)
                .load_collection::<SpriteAssets>()
                .load_collection::<SpriteSheetLayouts>()
                .load_collection::<FontAssets>(),
        );
    }
}
//dead code, not used anywhere
#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "font.otf")]
    pub game_font: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct SpriteSheetLayouts {
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 13, rows = 21))]
    pub player_atlas_layout: Handle<TextureAtlasLayout>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 32, columns = 5, rows = 1))]
    pub fireball_layout: Handle<TextureAtlasLayout>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 5, rows = 1))]
    pub ice_bolt_layout: Handle<TextureAtlasLayout>,
}

#[derive(AssetCollection, Resource)]
pub struct SpriteAssets {
    #[asset(path = "player/skeleton.png")]
    pub skeleton_player: Handle<Image>,
    #[asset(path = "items/sword.png")]
    pub sword_equipment_sprite: Handle<Image>,
    #[asset(path = "items/fire_staff.png")]
    pub fire_staff: Handle<Image>,
    #[asset(path = "items/ice_staff.png")]
    pub ice_staff: Handle<Image>,
    #[asset(path = "items/helmet.png")]
    pub helmet_equipment_sprite: Handle<Image>,
    #[asset(path = "items/shovel.png")]
    pub shovel_equipment_sprite: Handle<Image>,
    #[asset(path = "enemies/merman.png")]
    pub merman_enemy: Handle<Image>,
    #[asset(path = "projectiles/IceBolt.png")]
    pub ice_bolt: Handle<Image>,
    #[asset(path = "projectiles/fireball.png")]
    pub fire_ball: Handle<Image>,
    #[asset(path = "warpzone.png")]
    pub warp_zone: Handle<Image>,
    #[asset(path = "tilesets/tiles.png")]
    pub tiles: Handle<Image>,
    #[asset(path = "tilesets/grass_tileset.png")]
    pub grass_tiles: Handle<Image>,
    #[asset(path = "tilesets/water_tileset.png")]
    pub water_tiles: Handle<Image>,
    #[asset(path = "tilesets/wall_tileset.png")]
    pub wall_tiles: Handle<Image>,
    #[asset(path = "tilesets/wood_tileset.png")]
    pub wood_tiles: Handle<Image>,
    #[asset(path = "run_start_portal.png")]
    pub run_start_portal: Handle<Image>,
    #[asset(path = "npc.png")]
    pub npc: Handle<Image>,
    #[asset(path = "open_chest.png")]
    pub open_chest: Handle<Image>,
    #[asset(path = "closed_chest.png")]
    pub closed_chest: Handle<Image>,
    #[asset(path = "player/player_sprite_sheet.png")]
    pub player_sprite_sheet: Handle<Image>,
}

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
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 13, rows = 21))]
    pub enemy_atlas_layout: Handle<TextureAtlasLayout>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 32, columns = 5, rows = 1))]
    pub fireball_layout: Handle<TextureAtlasLayout>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 5, rows = 1))]
    pub ice_bolt_layout: Handle<TextureAtlasLayout>,
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 4, rows = 4))]
    pub bat_enemy_layout: Handle<TextureAtlasLayout>,
}

#[derive(AssetCollection, Resource)]
pub struct SpriteAssets {
    #[asset(path = "player/skeleton.png")]
    pub skeleton_player: Handle<Image>,
    #[asset(path = "items/sword.png")]
    pub sword: Handle<Image>,
    #[asset(path = "items/axe.png")]
    pub axe: Handle<Image>,
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
    #[asset(path = "tilesets/ground_tiles.png")]
    pub ground_tiles: Handle<Image>,
    #[asset(path = "tilesets/grass_tiles.png")]
    pub grass_tiles: Handle<Image>,
    #[asset(path = "tilesets/water_tiles.png")]
    pub water_tiles: Handle<Image>,
    #[asset(path = "tilesets/wall_tiles.png")]
    pub wall_tiles: Handle<Image>,
    #[asset(path = "tilesets/wood_tiles.png")]
    pub wood_tiles: Handle<Image>,
    #[asset(path = "door.png")]
    pub run_start_portal: Handle<Image>,
    #[asset(path = "open_chest.png")]
    pub open_chest: Handle<Image>,
    #[asset(path = "closed_chest.png")]
    pub closed_chest: Handle<Image>,
    #[asset(path = "player/player_sprite_sheet.png")]
    pub player_sprite_sheet: Handle<Image>,
    #[asset(path = "enemies/basic_enemy_sprite_sheet.png")]
    pub enemy_sprite_sheet: Handle<Image>,
    #[asset(path = "npcs/shop_keeper.png")]
    pub shop_keeper_sprite_sheet: Handle<Image>,
    #[asset(path = "npcs/game_guide.png")]
    pub game_guide_sprite_sheet: Handle<Image>,
    #[asset(path = "npcs/stat_trainer.png")]
    pub stat_trainer_sprite_sheet: Handle<Image>,
}

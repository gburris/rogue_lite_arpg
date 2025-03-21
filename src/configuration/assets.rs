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
                .load_collection::<GameIcons>(),
        );
    }
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
    #[asset(texture_atlas_layout(
        tile_size_x = 48,
        tile_size_y = 24,
        columns = 5,
        rows = 8,
        padding_y = 8,
        offset_y = 8,
    ))]
    pub chest_layout: Handle<TextureAtlasLayout>,
    #[asset(texture_atlas_layout(tile_size_x = 100, tile_size_y = 100, columns = 10, rows = 1))]
    pub spell_effect: Handle<TextureAtlasLayout>,
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 4, rows = 1))]
    pub shield_layout: Handle<TextureAtlasLayout>,
}

#[derive(AssetCollection, Resource)]
pub struct GameIcons {
    #[asset(path = "icons/equip_marker.png")]
    pub equip_icon: Handle<Image>,
    #[asset(path = "icons/potion.png")]
    pub potion_icon: Handle<Image>,
    #[asset(path = "icons/spell-book.png")]
    pub spell_book_icon: Handle<Image>,
    #[asset(path = "icons/sword-brandish.png")]
    pub melee_icon: Handle<Image>,
    #[asset(path = "icons/wizard-staff.png")]
    pub staff_icon: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct SpriteAssets {
    #[asset(path = "coin.png")]
    pub gold_coin: Handle<Image>,
    #[asset(path = "items/tome_of_healing.png")]
    pub tome_of_healing: Handle<Image>,
    #[asset(path = "items/knight_shield.png")]
    pub knight_shield: Handle<Image>,
    #[asset(path = "items/magic_shield.png")]
    pub magic_shield: Handle<Image>,
    #[asset(path = "items/sword.png")]
    pub sword: Handle<Image>,
    #[asset(path = "items/axe.png")]
    pub axe: Handle<Image>,
    #[asset(path = "items/fire_staff.png")]
    pub fire_staff: Handle<Image>,
    #[asset(path = "items/ice_staff.png")]
    pub ice_staff: Handle<Image>,
    #[asset(path = "items/health_potion.png")]
    pub health_potion: Handle<Image>,
    #[asset(path = "projectiles/IceBolt.png")]
    pub ice_bolt: Handle<Image>,
    #[asset(path = "projectiles/fireball.png")]
    pub fire_ball: Handle<Image>,
    #[asset(path = "door.png")]
    pub exit_door: Handle<Image>,
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
    #[asset(path = "tilesets/cobblestone_tiles.png")]
    pub cobblestone_tiles: Handle<Image>,
    #[asset(path = "door.png")]
    pub run_start_door: Handle<Image>,
    #[asset(path = "chests.png")]
    pub chests_sprite_sheet: Handle<Image>,
    #[asset(path = "spells/tome_of_healing_effect.png")]
    pub tome_of_healing_effect: Handle<Image>,
    #[asset(path = "player/player_sprite_sheet.png")]
    pub player_sprite_sheet: Handle<Image>,
    #[asset(path = "enemies/ice_mage_enemy.png")]
    pub ice_mage_enemy_sprite_sheet: Handle<Image>,
    #[asset(path = "enemies/warrior_enemy.png")]
    pub warrior_enemy_sprite_sheet: Handle<Image>,
    #[asset(path = "enemies/fire_mage_enemy.png")]
    pub fire_mage_enemy_sprite_sheet: Handle<Image>,
    #[asset(path = "npcs/shop_keeper.png")]
    pub shop_keeper_sprite_sheet: Handle<Image>,
    #[asset(path = "npcs/game_guide.png")]
    pub game_guide_sprite_sheet: Handle<Image>,
    #[asset(path = "npcs/stat_trainer.png")]
    pub stat_trainer_sprite_sheet: Handle<Image>,
}

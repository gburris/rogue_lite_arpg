use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::labels::states::AppState;

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .continue_to_state(AppState::CreateOverworld)
                .load_collection::<SpriteAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "font.otf")]
    pub game_font: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct SpriteAssets {
    #[asset(path = "font.otf")]
    pub game_font: Handle<Font>,
    #[asset(path = "skeleton.png")]
    pub skeleton_player: Handle<Image>,
    #[asset(path = "sword.png")]
    pub sword_equipment_sripte: Handle<Image>,
    #[asset(path = "helmet.png")]
    pub helmet_equipment_sripte: Handle<Image>,
    #[asset(path = "shovel.png")]
    pub shovel_equipment_sprite: Handle<Image>,
    #[asset(path = "merman.png")]
    pub merman_enemy: Handle<Image>,
    #[asset(path = "merman_on_fire.png")]
    pub merman_on_fire: Handle<Image>,
    #[asset(path = "merman_freezing.png")]
    pub merman_freezing: Handle<Image>,
    #[asset(path = "projectiles/IceBolt.png")]
    pub ice_bolt: Handle<Image>,
    #[asset(path = "projectiles/FB001.png")]
    pub fire_bolt: Handle<Image>,
    #[asset(path = "warpzone.png")]
    pub warp_zone: Handle<Image>,
    #[asset(path = "tiles.png")]
    pub tiles: Handle<Image>,
    #[asset(path = "grass_tileset.png")]
    pub grass_tiles: Handle<Image>,
    #[asset(path = "run_start_portal.png")]
    pub run_start_portal: Handle<Image>,
    #[asset(path = "npc.png")]
    pub npc: Handle<Image>,
}

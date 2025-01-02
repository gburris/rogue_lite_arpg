use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::labels::states::GameState;

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::CreateLevel)
                .load_collection::<SpriteAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct SpriteAssets {
    #[asset(path = "skeleton.png")]
    pub skeleton_player: Handle<Image>,
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
}

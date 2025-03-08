mod constants;
mod damage_overlay;
mod display_case;
mod display_case_slot;
mod game_over_screen;
mod input;
mod loading;
mod menu_helpers;
mod npc;
mod pause_menu;
mod player_overlay;
pub mod plugin;
mod start_screen;

// Be intentional about what we expose outside the module
// These are marker components for use in our despawn logic
// Otherwise, UI code should not need to be called outside the UI module 99% of the time
pub use game_over_screen::GameOverScreen;
pub use npc::stats_shop::StatShopMenu;
pub use pause_menu::inventory_menu::InventoryMenu;
pub use pause_menu::main_menu::MainMenu;
pub use pause_menu::plugin::PauseMenuPlugin;
pub use pause_menu::stats_menu::StatsMenu;
pub use player_overlay::PlayerOverlay;

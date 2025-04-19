mod constants;
mod damage_overlay;
mod display_case;
mod display_case_slot;
mod game_over_screen;
mod input;
mod load_screen;
mod npc;
mod pause_menu;
mod player_overlay;
pub mod plugin;
pub mod primitives;
mod start_screen;

// Be intentional about what we expose outside the module
// These are marker components for use in our despawn logic
// Otherwise, UI code should not need to be called outside the UI module 99% of the time
pub use player_overlay::PlayerOverlay;

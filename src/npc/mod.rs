pub mod components;
pub mod despawn;
pub mod dialouge;
pub mod events;
pub mod movement;
pub mod plugin;
pub mod setup;

pub use components::NPC;
pub use despawn::despawn_all_npcs;
pub use dialouge::begin_dialogue;
pub use dialouge::handle_dialogue_input;
pub use dialouge::update_dialogue_bubbles;
pub use events::DespawnAllNPCs;
pub use movement::move_npcs;
pub use movement::NPCMovement;
pub use plugin::NPCPlugin;
pub use setup::npc_setup;

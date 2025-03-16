pub mod health;
pub mod mana;

pub use health::on_healing_event;
pub use health::Health;
pub use mana::{Mana, ManaDrainRate};

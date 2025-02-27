pub mod health;
pub mod mana;

// Allows import of "use crate::combat::attributes::Health"
pub use health::on_healing_event;
pub use health::Health;
pub use mana::Mana;

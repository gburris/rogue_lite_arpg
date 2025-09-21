pub mod damage;
pub mod health;
pub mod invulnerable;
pub mod mana;
pub mod melee;
pub mod plugin;
pub mod projectile;
pub mod shield;
pub mod status_effects;

// These exist just to reduce stutter
pub use health::Health;
pub use mana::Mana;
pub use projectile::Projectile;

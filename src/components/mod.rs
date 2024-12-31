// src/components/mod.rs

pub mod burning_effect;
pub mod collider;
pub mod damage_effect;
pub mod enemy;
pub mod fireball;
pub mod player; // Declare the player module
pub mod position; // Declare the enemy module

pub mod health_bar;
pub mod health_text;
pub mod projectile;

pub use burning_effect::BurningEffect;
pub use collider::Collider;
pub use damage_effect::DamageEffect;
pub use health_bar::HealthBar;
pub use health_text::HealthText;
pub use projectile::Projectile;
// Re-export the components so they can be easily accessed
pub use enemy::*;
pub use fireball::*;
pub use player::*;
pub use position::*;

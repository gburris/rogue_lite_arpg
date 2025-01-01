// src/components/mod.rs

pub mod burning_effect;
pub mod collider;
pub mod damage_effect;
pub mod enemy;
pub mod fireball;
pub mod freezing_effect;
pub mod icebolt;
pub mod player; // Declare the enemy module
pub mod health;
pub mod health_bar;
pub mod projectile;
pub mod speed;

pub use speed::Speed;
pub use burning_effect::BurningEffect;
pub use collider::Collider;
pub use damage_effect::DamageEffect;
pub use health::Health;
pub use health_bar::HealthBar;
pub use projectile::Projectile;
// Re-export the components so they can be easily accessed
pub use enemy::*;
pub use fireball::*;
pub use freezing_effect::*;
pub use icebolt::*;
pub use player::*;

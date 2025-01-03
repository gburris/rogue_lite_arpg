pub mod components;
pub mod despawn;
pub mod handle_projectile_hit;
pub mod plugin;

pub use components::*;
pub use handle_projectile_hit::handle_projectile_hit;
pub use plugin::ProjectilePlugin;

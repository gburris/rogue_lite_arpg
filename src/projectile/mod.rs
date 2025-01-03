pub mod components;
pub mod despawn;
pub mod handle_projectile_hit;
pub mod plugin;
pub mod projectile_hit_event;

pub use components::*;
pub use handle_projectile_hit::handle_projectile_hit;
pub use plugin::ProjectilePlugin;
pub use projectile_hit_event::*;

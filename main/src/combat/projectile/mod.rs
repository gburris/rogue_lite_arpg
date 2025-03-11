pub mod components;
mod handle_collisions;
mod projectile_weapon;
pub mod spawn;

pub use handle_collisions::handle_projectile_collisions;
pub use projectile_weapon::ProjectileWeapon;

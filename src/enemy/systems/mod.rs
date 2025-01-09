pub mod despawn_enemy;
pub mod enemy_movement;
pub mod enemy_spawn;
pub mod handle_damage;
pub mod handle_enemy_defeated;

pub use despawn_enemy::despawn_on_zero_health;
pub use enemy_movement::move_enemies_toward_player;
pub use enemy_spawn::spawn_enemies_with_timer;
pub use handle_damage::despawn_all_enemies;
pub use handle_damage::*;
pub use handle_enemy_defeated::handle_enemy_defeated;

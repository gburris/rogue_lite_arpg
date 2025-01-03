use bevy::prelude::*;
#[derive(Component)]
pub struct StatusEffects {
    pub effects: Vec<StatusEffect>,
}
//Implement a default for status effects with an  empty effect vector
impl Default for StatusEffects {
    fn default() -> Self {
        StatusEffects {
            effects: Vec::new(),
        }
    }
}
#[derive(Clone)]
pub struct StatusEffect {
    pub effect_type: StatusEffectType,
    pub duration: Timer,
    pub damage_per_second: f32,
}

#[derive(Clone)]
pub enum StatusEffectType {
    Burning,
    Slowed,
    Stunned,
}

#[derive(Component)]
pub struct FreezingEffect {
    pub duration: Timer,
    pub slow_percentage: f32,
}

#[derive(Component)]
pub struct BurningEffect {
    pub damage_per_second: f32,
    pub duration: Timer,
    pub tick_timer: Timer,
}

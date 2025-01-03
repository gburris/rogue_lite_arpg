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

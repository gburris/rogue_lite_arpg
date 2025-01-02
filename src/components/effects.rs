use bevy::prelude::*;
#[derive(Component)]
pub struct StatusEffects {
    pub effects: Vec<Effect>,
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
pub struct Effect {
    pub effect_type: EffectType,
    pub duration: Timer,
    pub damage_per_second: f32,
}

#[derive(Clone)]
pub enum EffectType {
    Burning,
    Slowed,
    Stunned,
}

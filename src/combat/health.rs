use bevy::prelude::*;

#[derive(EntityEvent)]
pub struct AttemptHeal {
    pub entity: Entity,
    pub amount: f32,
}

#[derive(EntityEvent)]
pub struct Healed {
    pub entity: Entity,
    pub amount: f32,
}

#[derive(Component)]
pub struct Health {
    pub hp: f32,
    pub max_hp: f32,
}

impl Health {
    pub fn new(max_hp: f32) -> Self {
        Self { hp: max_hp, max_hp }
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.hp = (self.hp - amount).max(0.0);
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }

    /// Current health as a `0.0..=1.0` fraction, or `None` when the health
    /// values cannot describe a valid fraction.
    pub fn fraction(&self) -> Option<f32> {
        if self.hp.is_finite() && self.max_hp.is_finite() && self.max_hp > 0.0 {
            Some((self.hp / self.max_hp).clamp(0.0, 1.0))
        } else {
            None
        }
    }

    fn add_health(&mut self, amount: f32) -> f32 {
        let before = self.hp;
        self.hp += amount;
        if self.hp > self.max_hp {
            self.hp = self.max_hp;
        }
        self.hp - before
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(100.0)
    }
}

pub(super) fn on_healing_event(
    attempt_heal: On<AttemptHeal>,
    mut commands: Commands,
    mut healed_query: Query<&mut Health>,
) {
    if let Ok(mut health) = healed_query.get_mut(attempt_heal.entity) {
        let actual_amount = health.add_health(attempt_heal.amount);
        commands.trigger(Healed {
            entity: attempt_heal.entity,
            amount: actual_amount,
        });
        let entity = attempt_heal.entity;
        info!("Entity {entity} healed by {actual_amount:.2} points");
    }
}

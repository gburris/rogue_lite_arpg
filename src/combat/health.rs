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
        self.hp -= amount;
        if self.hp < 0.0 {
            self.hp = 0.0;
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
        Health {
            hp: 100.0,
            max_hp: 100.0,
        }
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
        info!(
            "Entity {} healed by {:.2} points",
            attempt_heal.entity, actual_amount,
        );
    }
}

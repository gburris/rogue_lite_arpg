use bevy::prelude::*;

#[derive(Event)]
pub struct AttemptHealingEvent {
    pub amount: f32,
}

#[derive(Event)]
pub struct HealedEvent {
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

pub fn on_healing_event(
    healing_trigger: Trigger<AttemptHealingEvent>,
    mut commands: Commands,
    mut healed_query: Query<&mut Health>,
) {
    if let Ok(mut health) = healed_query.get_mut(healing_trigger.target()) {
        let actual_amount = health.add_health(healing_trigger.amount);
        commands.trigger_targets(
            HealedEvent {
                amount: actual_amount,
            },
            healing_trigger.target(),
        );
        info!(
            "Entity {} healed by {:.2} points",
            healing_trigger.target(),
            actual_amount,
        );
    }
}

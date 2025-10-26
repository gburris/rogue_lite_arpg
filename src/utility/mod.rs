use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (despawn_expired_entities, generic_remove_component_system)
            .in_set(InGameSystems::DespawnEntities),
    );
}

/// Represents an entity that will be despawned after time elapsed
#[derive(Component, Clone)]
pub struct Lifespan(pub Timer);

impl Lifespan {
    pub fn new(duration_secs: f32) -> Self {
        Lifespan(Timer::from_seconds(duration_secs, TimerMode::Once))
    }
}

impl Default for Lifespan {
    fn default() -> Self {
        Lifespan(Timer::from_seconds(2.0, TimerMode::Once))
    }
}

fn despawn_expired_entities(
    mut commands: Commands,
    mut duration_query: Query<(Entity, &mut Lifespan)>,
    time: Res<Time>,
) {
    for (entity, mut duration) in duration_query.iter_mut() {
        duration.0.tick(time.delta());

        if duration.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Despawn all entities with the specific component
pub fn despawn_all<T: Event, C: Component>(
    _: On<T>,
    mut commands: Commands,
    query: Query<Entity, With<C>>,
) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}

#[derive(Component)]
struct RemoveComponent {
    timer: Timer,
    remover: Option<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
}

fn generic_remove_component_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut RemoveComponent)>,
    time: Res<Time>,
) {
    for (entity, mut remove_component) in query.iter_mut() {
        remove_component.timer.tick(time.delta());
        if remove_component.timer.is_finished()
            && let Some(remover) = remove_component.remover.take()
        {
            let mut entity_cmds = commands.entity(entity);
            remover(&mut entity_cmds);
            entity_cmds.remove::<RemoveComponent>();
        }
    }
}

pub fn schedule_component_removal<C: Component>(
    commands: &mut Commands,
    entity: Entity,
    seconds: f32,
) {
    let remover = Box::new(|entity_cmds: &mut EntityCommands| {
        entity_cmds.remove::<C>();
    });

    commands.entity(entity).insert(RemoveComponent {
        timer: Timer::from_seconds(seconds, TimerMode::Once),
        remover: Some(remover),
    });
}

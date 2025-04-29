use std::any::TypeId;

use bevy::{
    ecs::component::{ComponentId, Components},
    prelude::*,
};
use bevy_ecs_tilemap::map::TilemapId;

use crate::{
    combat::Projectile,
    configuration::time_control::RestartEvent,
    economy::Gold,
    items::lootable::Lootable,
    labels::sets::InGameSet,
    map::{portal::Portal, systems::zone::ZoneBackground, Chest, CleanupZone, Wall, Water},
    prelude::{Enemy, Player, NPC},
    ui::PlayerOverlay,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (despawn_expired_entities).in_set(InGameSet::DespawnEntities),
    )
    .add_observer(despawn_all::<CleanupZone, Portal>)
    .add_observer(despawn_all::<CleanupZone, TilemapId>)
    .add_observer(despawn_all::<CleanupZone, Wall>)
    .add_observer(despawn_all::<CleanupZone, Water>)
    .add_observer(despawn_all::<CleanupZone, ZoneBackground>)
    .add_observer(despawn_all::<CleanupZone, Lootable>)
    .add_observer(despawn_all::<CleanupZone, Chest>)
    .add_observer(despawn_all::<CleanupZone, Enemy>)
    .add_observer(despawn_all::<CleanupZone, Projectile>)
    .add_observer(despawn_all::<CleanupZone, NPC>)
    .add_observer(despawn_all::<CleanupZone, Gold>)
    .add_observer(despawn_all::<RestartEvent, Player>)
    .add_observer(despawn_all::<RestartEvent, PlayerOverlay>);
}

/// Represents an entity that will be despawned after time elapsed
#[derive(Component)]
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

pub fn despawn_expired_entities(
    mut commands: Commands,
    mut duration_query: Query<(Entity, &mut Lifespan)>,
    time: Res<Time>,
) {
    for (entity, mut duration) in duration_query.iter_mut() {
        duration.0.tick(time.delta());

        if duration.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Despawn all entities with the specific component
pub fn despawn_all<T: Event, C: Component>(
    _: Trigger<T>,
    mut commands: Commands,
    query: Query<Entity, With<C>>,
) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}

#[derive(Component)]
pub struct RemoveComponent {
    pub timer: Timer,
    pub component_id: ComponentId,
}

pub fn generic_remove_component_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut RemoveComponent)>,
    time: Res<Time>,
) {
    for (entity, mut remove_component) in query.iter_mut() {
        remove_component.timer.tick(time.delta());
        if remove_component.timer.finished() {
            commands
                .entity(entity)
                .remove_by_id(remove_component.component_id)
                .remove::<RemoveComponent>();
        }
    }
}

pub fn schedule_component_removal<C: Component>(
    commands: &mut Commands,
    entity: Entity,
    seconds: f32,
    components: &Components,
) {
    let component_id = components
        .get_id(TypeId::of::<C>())
        .expect("No component of type T registered");

    commands.entity(entity).insert(RemoveComponent {
        timer: Timer::from_seconds(seconds, TimerMode::Once),
        component_id,
    });
}

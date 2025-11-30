use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_lit::prelude::PointLight2d;
use rand::{Rng, rng};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, glow_and_rotate_lootables.in_set(InGameSystems::Vfx));

    app.add_observer(on_item_added_make_lootable)
        .add_observer(on_drop_event)
        .add_observer(despawn_all::<CleanupZone, Lootable>);
}

#[derive(Component, Clone, Debug, Default)]
#[require(
    Lifespan::new(10.0),
    YSort::from_offset(-6.0),
    PointLight2d {
                intensity: 0.3,
                color: Color::WHITE,
                ..default()
    },
)]
pub struct Lootable;

#[derive(EntityEvent)]
pub struct ItemDrop {
    pub entity: Entity,
}

/// Notes:
/// 1. `ItemDropEvent` is for items only!
/// 2. This event will handle unequipping and removing any items dropped from the inventory of the holder
/// 3. Needs parent to be holder for position, then removes parent
fn on_drop_event(
    item_dropped: On<ItemDrop>,
    mut commands: Commands,
    item_query: Query<&ItemOf>,
    mut holder_query: Query<&Transform, With<Items>>,
) {
    let item_entity = item_dropped.entity;

    let Ok(ItemOf(holder_entity)) = item_query.get(item_entity) else {
        warn!("Lootable item missing parent");
        return;
    };

    let Ok(parent_transform) = holder_query.get_mut(*holder_entity) else {
        error!("Why does the parent not have a transform or items on drop");
        return;
    };

    // TODO: Make sure we don't drop items out of bounds
    let mut rng = rng();
    let offset = Vec2::new(rng.random_range(-50.0..50.0), rng.random_range(-50.0..50.0));
    let final_position =
        (parent_transform.translation.truncate() + offset).extend(ZLayer::OnGround.z());

    trace!("Dropping item at {}", offset);

    commands.trigger(Unequip {
        entity: item_entity,
    });

    commands
        .entity(item_entity)
        .remove::<ItemOf>()
        .insert((
            Lootable,
            PointLight2d {
                intensity: 0.3,
                color: Color::WHITE,
                ..default()
            },
            Visibility::Visible,
            Transform::from_translation(final_position),
        ))
        .with_child(InteractionZone::ITEM_PICKUP);
}

fn on_lootable_item_interaction(
    interaction: On<PlayerInteraction>,
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
) {
    let item_entity = interaction.entity;

    // Make sure item doesn't despawn and is hidden (since its in inventory)
    commands
        .entity(item_entity)
        .remove::<(YSort, Lifespan, Lootable, PointLight2d)>()
        .insert((ItemOf(*player), Visibility::Hidden));

    // Remove interaction zone once itme is picked up
    commands
        .entity(interaction.interaction_zone_entity)
        .despawn();
}

fn glow_and_rotate_lootables(
    mut query: Query<(&mut Item, &mut Transform, &mut Sprite), With<Lootable>>,
    time: Res<Time>,
) {
    for (mut item, mut transform, mut sprite) in &mut query {
        item.drop_rotation_timer += time.delta_secs();
        let rotation_angle = (item.drop_rotation_timer / 6.0) * 2.0 * PI;
        transform.rotation = Quat::from_rotation_z(rotation_angle);
        item.drop_glow_effect += time.delta_secs() * 2.0;
        let glow_intensity = item.drop_glow_effect.sin() * 0.1 + 0.7;
        let base_color = sprite.color.to_srgba();
        sprite.color = Color::srgba(
            base_color.red * glow_intensity + 0.3,
            base_color.green * glow_intensity + 0.3,
            base_color.blue * glow_intensity + 0.3,
            base_color.alpha,
        );
    }
}

fn on_item_added_make_lootable(item_added: On<Add, Item>, mut commands: Commands) {
    // We do this to avoid having to manually add this observer to every item we create
    commands
        .entity(item_added.entity)
        .observe(on_lootable_item_interaction);
}

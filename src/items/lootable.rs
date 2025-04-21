use std::f32::consts::PI;

use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    configuration::{YSort, ZLayer},
    items::equipment::Equipped,
    player::{
        interact::{InteractionEvent, InteractionZone},
        Player,
    },
    utility::Lifespan,
};

use super::{inventory::Inventory, Item};

#[derive(Component, Clone, Debug, Default)]
#[require(
    Lifespan::new(10.0),
    YSort::from_offset(-6.0)
)]
pub struct Lootable;

#[derive(Event)]
pub struct ItemDropEvent;

/// Notes:
/// 1. ItemDropEvent is for items only!
/// 2. This event will handle unequipping and removing any items dropped from the inventory of the holder
/// 3. Needs parent to be holder for position, then removes parent
pub fn on_drop_event(
    trigger: Trigger<ItemDropEvent>,
    mut commands: Commands,
    item_query: Query<&ChildOf, With<Item>>,
    mut parent_query: Query<(&Transform, &mut Inventory)>,
) {
    let item_entity = trigger.target();

    let Ok(child_of) = item_query.get(item_entity) else {
        warn!("Lootable item missing parent");
        return;
    };

    let Ok((parent_transform, mut inventory)) = parent_query.get_mut(child_of.parent) else {
        error!("Why does the parent not have a transform or inventory on drop");
        return;
    };

    let mut rng = thread_rng();
    let offset = Vec2::new(rng.gen_range(-50.0..50.0), rng.gen_range(-50.0..50.0));
    let final_position =
        (parent_transform.translation.truncate() + offset).extend(ZLayer::OnGround.z());

    // We don't care if item is actually found in inventory
    inventory.remove_item(item_entity).ok();

    trace!("Dropping item at {}", offset);

    commands
        .entity(item_entity)
        .remove::<Equipped>()
        .insert((
            Lootable,
            Visibility::Visible,
            Transform::from_translation(final_position),
        ))
        .remove::<ChildOf>()
        .with_child(InteractionZone::ITEM_PICKUP);
}

pub fn on_lootable_item_interaction(
    trigger: Trigger<InteractionEvent>,
    mut commands: Commands,
    player: Single<(Entity, &mut Inventory), With<Player>>,
) {
    let item_entity = trigger.target();

    let (player_entity, mut inventory) = player.into_inner();

    if inventory.add_item(item_entity).is_ok() {
        commands.entity(player_entity).add_child(item_entity);

        // Make sure item doesn't despawn and is hidden (since its in inventory)
        commands
            .entity(item_entity)
            .remove::<Lootable>()
            .remove::<Lifespan>()
            .remove::<YSort>()
            .insert(Visibility::Hidden);

        // Remove interaction zone once itme is picked up
        commands.entity(trigger.interaction_zone_entity).despawn();
    } else {
        warn!("Inventory is full!")
    }
}

pub fn glow_and_rotate_lootables(
    mut query: Query<(&mut Item, &mut Transform, &mut Sprite), With<Lootable>>,
    time: Res<Time>,
) {
    for (mut item, mut transform, mut sprite) in query.iter_mut() {
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

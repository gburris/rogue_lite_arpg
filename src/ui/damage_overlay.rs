use avian2d::prelude::ColliderAabb;
use bevy::prelude::*;
use rand::RngExt;

use crate::prelude::{DamageDealt, Healed, Health, HurtBox, Lifespan, ZLayer};

const RED_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const GREEN_COLOR: Color = Color::srgb(0.0, 0.8, 0.0);
const HEALTH_TEXT_OFFSET: f32 = 10.0;

// Characters don't have a collider on their root entity, so we use their hurtbox to determine
// height
fn entity_height(
    entity: Entity,
    children_query: &Query<&Children>,
    hurt_box_query: &Query<&ColliderAabb, With<HurtBox>>,
) -> f32 {
    children_query
        .get(entity)
        .ok()
        .and_then(|children| hurt_box_query.iter_many(children).next())
        // Assume the entity is 32 pixels tall if no hurtbox can be found.
        .map_or(32.0, |aabb| aabb.max.y - aabb.min.y)
}

/// Text is placed relative to `origin`, which is the center of the entity the text is for.
fn health_change_text(origin: Vec3, entity_height: f32, amount: f32, color: Color) -> impl Bundle {
    // Create a quaternion for the random rotation
    let random_rotation = Quat::from_axis_angle(Vec3::Z, random_angle(30.0));

    // Get rotation assuming sprite is facing "UP" (y axis)
    let rotated_vector = (random_rotation * Vec3::Y).truncate();

    // Text height is relative to center of entity, so we get half of entity height and add a buffer
    let text_height = (entity_height / 2.0) + HEALTH_TEXT_OFFSET;

    // Scale the direction vector by the desired text height to place the text above the entity
    let text_position = (rotated_vector.normalize() * text_height).extend(ZLayer::AboveSprite.z());

    let rounded_amount = (amount * 10.0).round() / 10.0; // Round to 1 decimal place
    let formatted_amount = if rounded_amount.fract() == 0.0 {
        format!("{rounded_amount:.0}") // Display as a whole number
    } else {
        format!("{rounded_amount:.1}") // Display with one decimal place
    };

    (
        Text2d::new(formatted_amount),
        TextColor::from(color),
        Lifespan::new(0.4),
        Transform::from_translation(origin + text_position),
    )
}

// Generate a random angle between -angle_range and angle_range degrees (convert to radians)
fn random_angle(angle_range: f32) -> f32 {
    let mut rng = rand::rng();
    rng.random_range(-angle_range..angle_range).to_radians()
}

pub fn on_damage_overlay_amount(
    damage_dealt: On<DamageDealt>,
    mut commands: Commands,
    damaged_query: Query<(&Health, &Transform)>,
    children_query: Query<&Children>,
    hurt_box_query: Query<&ColliderAabb, With<HurtBox>>,
) {
    let entity = damage_dealt.entity;
    let entity_height = entity_height(entity, &children_query, &hurt_box_query);

    match damaged_query.get(entity) {
        // Defeated characters have their children despawned, so text for the killing blow
        // is spawned in world space rather than following the entity
        Ok((health, transform)) if health.is_dead() => {
            commands.spawn(health_change_text(
                transform.translation,
                entity_height,
                damage_dealt.damage,
                RED_COLOR,
            ));
        }
        _ => {
            commands.entity(entity).with_child(health_change_text(
                Vec3::ZERO,
                entity_height,
                damage_dealt.damage,
                RED_COLOR,
            ));
        }
    }
}

pub fn on_healing_overlay_amount(
    healed: On<Healed>,
    mut commands: Commands,
    children_query: Query<&Children>,
    hurt_box_query: Query<&ColliderAabb, With<HurtBox>>,
) {
    let entity_height = entity_height(healed.entity, &children_query, &hurt_box_query);

    commands
        .entity(healed.entity)
        .with_child(health_change_text(
            Vec3::ZERO,
            entity_height,
            healed.amount,
            GREEN_COLOR,
        ));
}

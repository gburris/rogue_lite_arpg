use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_action_bar, update_cooldowns)
            .chain()
            .in_set(InGameSystems::HudOverlay),
    );

    app.add_observer(on_player_action_took_effect);
}

#[derive(Component, Clone, Default)]
pub(super) struct ActionBox {
    slot: EquipmentSlot,
}

impl ActionBox {
    fn key(&self) -> ActionKey {
        ActionKey::Equipped(self.slot)
    }
}

fn find_action_box(action_boxes: &Query<(Entity, &ActionBox)>, key: ActionKey) -> Option<Entity> {
    action_boxes
        .iter()
        .find(|(_, action_box)| action_box.key() == key)
        .map(|(box_entity, _)| box_entity)
}

#[derive(Component, Default, Clone)]
pub(super) struct CooldownIndicator;

#[derive(Component, Default, Clone)]
#[require(Lifespan::new(0.1))]
struct ErrorFlash;

const ACTION_BOX_SIZE: f32 = 50.0;
const ACTION_BOX_BORDER: f32 = 2.0;
const ACTION_BOX_INTERIOR_SIZE: f32 = ACTION_BOX_SIZE - (ACTION_BOX_BORDER * 2.0);
const ACTION_BOX_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8); // 80% opaque black
const ACTION_BOX_OUTLINE_COLOR: Color = Color::srgba(0.8, 0.8, 0.8, 0.5); // Semi-transparent white
const COOLDOWN_LINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.6); // Semi-transparent white
const ERROR_FLASH_COLOR: Color = Color::srgba(0.9, 0.2, 0.2, 0.2); // Semi-transparent red

pub(super) fn action_bar() -> impl Scene {
    bsn! {
        #ActionBar
        Node { flex_direction: FlexDirection::Row }
        Children [
            @action_box(EquipmentSlot::Mainhand)
            --
            @action_box(EquipmentSlot::Offhand)
        ]
    }
}

fn action_box(slot: EquipmentSlot) -> impl Scene {
    bsn! {
        ActionBox { slot }
        Node {
            width: px(ACTION_BOX_SIZE),
            height: px(ACTION_BOX_SIZE),
            border: px(ACTION_BOX_BORDER),
        }
        BackgroundColor(ACTION_BOX_COLOR)
        BorderColor::all(ACTION_BOX_OUTLINE_COLOR)
        Children [
            ImageNode { color: Color::NONE }
            Node {
                width: percent(100),
                height: percent(100),
            }
        ]
    }
}

/// Covers an action box's interior, for cooldown and error overlays.
fn action_box_overlay(box_entity: Entity, color: Color) -> impl Scene {
    bsn! {
        ChildOf(box_entity)
        Node {
            width: px(ACTION_BOX_INTERIOR_SIZE),
            height: px(ACTION_BOX_INTERIOR_SIZE),
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
        }
        BackgroundColor(color)
    }
}

/// Shows each slot's equipped item in its action box, and nothing in an empty slot.
fn update_action_bar(
    action_box_query: Query<(&ActionBox, &Children)>,
    mut image_query: Query<&mut ImageNode>,
    player_equipment: Single<(Option<&Mainhand>, Option<&Offhand>), With<Player>>,
    item_query: Query<&Sprite, With<Item>>,
) {
    let (mainhand, offhand) = *player_equipment;

    for (action_box, children) in &action_box_query {
        let Some(&image_entity) = children.first() else {
            continue;
        };
        let Ok(mut image_node) = image_query.get_mut(image_entity) else {
            continue;
        };
        let item_sprite = action_box
            .slot
            .equipped(mainhand, offhand)
            .and_then(|item| item_query.get(item).ok());
        let Some(item_sprite) = item_sprite else {
            // An image node with no image draws a solid square of its color.
            if image_node.color != Color::NONE {
                image_node.color = Color::NONE;
            }
            continue;
        };
        if image_node.color == Color::WHITE && image_node.image == item_sprite.image {
            continue;
        }
        let action_bar_sprite = action_bar_sprite(item_sprite);
        image_node.color = Color::WHITE;
        image_node.image = action_bar_sprite.image;
        image_node.texture_atlas = action_bar_sprite.texture_atlas;
    }
}

/// Adds a cooldown indicator to the action box of the slot whose use took effect.
fn on_player_action_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    players: Query<(), With<Player>>,
    action_box_query: Query<(Entity, &ActionBox)>,
    cooldowns: Query<&Cooldown>,
) {
    if !players.contains(took_effect.holder) {
        return;
    }
    if let Ok(cooldown) = cooldowns.get(took_effect.source)
        && let Some(box_entity) = find_action_box(&action_box_query, took_effect.key)
    {
        let cooldown = cooldown.remaining_seconds(time.elapsed_secs());
        commands.spawn_scene(bsn! {
            @action_box_overlay(box_entity, COOLDOWN_LINE_COLOR)
            CooldownIndicator
            Lifespan::new(cooldown)
        });
    }
}

pub(super) fn on_action_failed(
    failed: On<ActionFailed>,
    mut commands: Commands,
    action_box_query: Query<(Entity, &ActionBox)>,
) {
    let is_flashed = match failed.reason {
        ActionFailure::InProgress | ActionFailure::OnCooldown => false,
        ActionFailure::NoSource
        | ActionFailure::MissingRequiredItem
        | ActionFailure::HolderInterrupted
        | ActionFailure::OutOfMana => true,
    };
    if is_flashed && let Some(box_entity) = find_action_box(&action_box_query, failed.key) {
        commands.spawn_scene(bsn! {
            @action_box_overlay(box_entity, ERROR_FLASH_COLOR)
            ErrorFlash
        });
    }
}

pub(super) fn update_cooldowns(
    mut cooldown_query: Query<(&mut Node, &Lifespan), With<CooldownIndicator>>,
) {
    for (mut line_node, cooldown_duration) in &mut cooldown_query {
        line_node.height = px(ACTION_BOX_INTERIOR_SIZE * cooldown_duration.0.fraction_remaining());
    }
}

fn action_bar_sprite(sprite: &Sprite) -> Sprite {
    match &sprite.texture_atlas {
        Some(atlas) => Sprite {
            image: sprite.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: atlas.layout.clone(),
                index: 0,
            }),
            ..sprite.clone()
        },
        None => sprite.clone(),
    }
}

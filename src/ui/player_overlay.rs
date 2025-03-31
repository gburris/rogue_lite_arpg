use bevy::{ecs::spawn::SpawnIter, prelude::*};

use crate::{
    combat::{Health, Mana},
    despawn::components::LiveDuration,
    items::{
        equipment::{
            EquipmentSlot, EquipmentUseFailedEvent, EquipmentUseFailure, Equippable, Equipped,
            UseEquipmentEvent,
        },
        inventory::Inventory,
        Item,
    },
    player::Player,
};

#[derive(Component)]
pub struct PlayerOverlay;

#[derive(Component)]
pub struct ManaBar;

#[derive(Component)]
pub struct ManaLostBar {
    previous_mana: f32,
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthLostBar {
    previous_hp: f32,
}

#[derive(Component, Debug)]
pub struct ExpBar;

const EXP_COLOR: Color = Color::srgb(0.5, 0.0, 0.5);
const HEALTH_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const MANA_COLOR: Color = Color::srgb(0.0, 0.173, 0.878);
const BAR_CHANGE_COLOR: Color = Color::srgb(1.0, 0.89, 0.41);

// If health is 100, the health bar will be 400 pixels long. Same for mana.
const ATTRIBUTE_TO_PIXEL_SCALE: f32 = 4.0;

// Represents how fast the yellow "amount lost" of health or mana goes away
const LOST_AMOUNT_SHRINK_RATE: f32 = 80.0;

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        PlayerOverlay,
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        children![
            // Top left container for health and mana bars
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                children![
                    attribute_bar(
                        HealthBar,
                        HealthLostBar { previous_hp: 100.0 },
                        HEALTH_COLOR,
                    ),
                    attribute_bar(
                        ManaBar,
                        ManaLostBar {
                            previous_mana: 100.0,
                        },
                        MANA_COLOR,
                    )
                ]
            ),
            Node {
                flex_grow: 1.0,
                ..default()
            },
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexEnd,
                    ..default()
                },
                children![experience_bar(), action_bar()]
            )
        ],
    ));
}

const ATTRIBUTE_BACKGROUND_COLOR: Color = Color::srgb(0.21, 0.21, 0.21);
const ATTRIBUTE_BAR_WIDTH: Val = Val::Px(400.0);

fn attribute_bar(
    marker_component: impl Component,
    change_component: impl Component,
    bar_color: Color,
) -> impl Bundle {
    (
        Node {
            width: ATTRIBUTE_BAR_WIDTH,
            height: Val::Px(15.0),
            ..default()
        },
        BackgroundColor::from(ATTRIBUTE_BACKGROUND_COLOR),
        children![
            (
                marker_component,
                Node {
                    width: ATTRIBUTE_BAR_WIDTH,
                    height: Val::Px(15.0),
                    ..default()
                },
                BackgroundColor::from(bar_color),
            ),
            (
                change_component,
                Node {
                    width: Val::Px(0.0),
                    height: Val::Px(15.0),
                    ..default()
                },
                BackgroundColor::from(BAR_CHANGE_COLOR),
            )
        ],
    )
}

pub fn update_health_bar(
    player_health: Option<Single<&Health, (With<Player>, Changed<Health>)>>,
    mut health_bar: Single<&mut Node, (With<HealthBar>, Without<HealthLostBar>)>,
    health_lost_bar: Single<(&mut Node, &mut HealthLostBar)>,
) {
    let (mut health_lost_node, mut health_lost) = health_lost_bar.into_inner();

    if let Some(player_health) = player_health {
        health_bar.width = get_amount_left_in_pixels(player_health.hp, player_health.max_hp);
        health_lost_node.width = get_amount_lost_in_pixels(
            health_lost.previous_hp,
            player_health.hp,
            health_lost_node.width,
        );

        health_lost.previous_hp = player_health.hp;
    }
}

pub fn update_mana_bar(
    player_mana: Option<Single<&Mana, (With<Player>, Changed<Mana>)>>,
    mut mana_bar: Single<&mut Node, (With<ManaBar>, Without<ManaLostBar>)>,
    mana_lost_bar: Single<(&mut Node, &mut ManaLostBar)>,
) {
    let (mut mana_lost_node, mut mana_lost) = mana_lost_bar.into_inner();

    if let Some(player_mana) = player_mana {
        mana_bar.width = get_amount_left_in_pixels(player_mana.current_mana, player_mana.max_mana);
        mana_lost_node.width = get_amount_lost_in_pixels(
            mana_lost.previous_mana,
            player_mana.current_mana,
            mana_lost_node.width,
        );

        mana_lost.previous_mana = player_mana.current_mana;
    }
}

pub fn update_lost_mana_bar(
    mut mana_lost_node: Single<&mut Node, With<ManaLostBar>>,
    time: Res<Time>,
) {
    let Val::Px(current_pixel) = mana_lost_node.width else {
        panic!("Non-pixel value for mana bar");
    };

    let amount_to_remove = LOST_AMOUNT_SHRINK_RATE * time.delta_secs();
    mana_lost_node.width = Val::Px((current_pixel - amount_to_remove).max(0.0));
}

pub fn update_lost_health_bar(
    mut health_lost_node: Single<&mut Node, With<HealthLostBar>>,
    time: Res<Time>,
) {
    let Val::Px(current_pixel) = health_lost_node.width else {
        panic!("Non-pixel value for mana bar");
    };

    let amount_to_remove = LOST_AMOUNT_SHRINK_RATE * time.delta_secs();
    health_lost_node.width = Val::Px((current_pixel - amount_to_remove).max(0.0));
}

// Gets length in Val::Px of bar representing amount of mana or health left
fn get_amount_left_in_pixels(current_amount: f32, max_amount: f32) -> Val {
    let max_bar_length = max_amount * ATTRIBUTE_TO_PIXEL_SCALE;
    let ratio_remaining = current_amount / max_amount;
    Val::Px(ratio_remaining * max_bar_length)
}

// Gets length in Val::Px of yellow bar representing amount of mana or health lost
fn get_amount_lost_in_pixels(previous_amount: f32, current_amount: f32, pixel_width: Val) -> Val {
    let pixel_change = (previous_amount - current_amount) * ATTRIBUTE_TO_PIXEL_SCALE;

    let Val::Px(current_pixels) = pixel_width else {
        panic!("Non-pixel value for amount lost bar");
    };

    // Negative pixel values arne't allowed
    Val::Px((current_pixels + pixel_change).max(0.0))
}

fn experience_bar() -> impl Bundle {
    (
        Node {
            width: Val::Px(400.0),
            height: Val::Px(20.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            // Add overflow visibility for debugging
            overflow: Overflow::visible(),
            ..default()
        },
        children![
            (
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor::from(ATTRIBUTE_BACKGROUND_COLOR),
            ),
            (
                ExpBar,
                Node {
                    width: Val::Px(0.0),
                    height: Val::Px(20.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor::from(EXP_COLOR),
                Children::spawn(SpawnIter((1..10).map(|i| (
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(i as f32 * 40.0),
                        width: Val::Px(2.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor::from(Color::srgba(1.0, 1.0, 1.0, 0.3)),
                ))))
            )
        ],
    )
}

pub fn update_exp_bar(
    player: Option<Single<&Player, Changed<Player>>>,
    mut exp_bar: Single<&mut Node, With<ExpBar>>,
) {
    if let Some(player) = player {
        exp_bar.width = Val::Px(400.0 * player.get_progress_to_next_level());
    }
}

/* Action Bar Components and Systems */
#[derive(Component)]
pub struct ActionBar;

#[derive(Component)]
pub struct ActionBox {
    slot: EquipmentSlot,
}

#[derive(Component)]
pub struct CooldownIndicator;

#[derive(Component)]
#[require(LiveDuration::new(0.1))]
pub struct ErrorFlash;

const ACTION_BOX_SIZE: f32 = 50.0;
const ACTION_BOX_BORDER: f32 = 2.0;
const ACTION_BOX_INTERIOR_SIZE: f32 = ACTION_BOX_SIZE - (ACTION_BOX_BORDER * 2.0);
const ACTION_BOX_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8); // 80% opaque black
const ACTION_BOX_OUTLINE_COLOR: Color = Color::srgba(0.8, 0.8, 0.8, 0.5); // Semi-transparent white
const COOLDOWN_LINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.6); // Semi-transparent white
const ERROR_FLASH_COLOR: Color = Color::srgba(0.9, 0.2, 0.2, 0.2); // Semi-transparent red

fn action_bar() -> impl Bundle {
    (
        ActionBar,
        Node {
            flex_direction: FlexDirection::Row,
            ..default()
        },
        Children::spawn(SpawnIter(
            Inventory::ALL_SLOTS.iter().map(|slot| action_box(*slot)),
        )),
    )
}

fn action_box(slot: EquipmentSlot) -> impl Bundle {
    (
        ActionBox { slot },
        Node {
            width: Val::Px(ACTION_BOX_SIZE),
            height: Val::Px(ACTION_BOX_SIZE),
            border: UiRect::all(Val::Px(ACTION_BOX_BORDER)),
            ..default()
        },
        BackgroundColor::from(ACTION_BOX_COLOR),
        BorderColor::from(ACTION_BOX_OUTLINE_COLOR),
        Children::spawn_one((
            ImageNode::default(),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        )),
    )
}

pub fn update_action_bar(
    action_box_query: Query<(&ActionBox, &Children)>,
    mut image_query: Query<&mut ImageNode>,
    inventory_query: Option<Single<&Inventory, (Changed<Inventory>, With<Player>)>>,
    item_query: Query<&Sprite, With<Item>>,
) {
    if let Some(player_inventory_result) = inventory_query {
        let player_inventory = player_inventory_result.into_inner();

        for (action_box, children) in action_box_query.iter() {
            if let Some(equipped_entity) = player_inventory.get_equipped(action_box.slot) {
                if let Some(&image_entity) = children.first() {
                    if let Ok(mut image_node) = image_query.get_mut(image_entity) {
                        if let Ok(item_sprite) = item_query.get(equipped_entity) {
                            let action_bar_sprite = get_action_bar_sprite(item_sprite);

                            image_node.image = action_bar_sprite.image.clone();

                            if let Some(atlas) = &action_bar_sprite.texture_atlas {
                                image_node.texture_atlas = Some(TextureAtlas {
                                    layout: atlas.layout.clone(),
                                    index: atlas.index,
                                });
                            } else {
                                image_node.texture_atlas = None;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn on_equipment_used(
    trigger: Trigger<UseEquipmentEvent>,
    player: Single<(Entity, &Player)>,
    mut commands: Commands,
    action_box_query: Query<(Entity, &ActionBox, &Children)>,
    equipment_query: Query<&Equippable, With<Equipped>>,
    error_flash_query: Query<Entity, With<ErrorFlash>>,
) {
    if trigger.holder != player.0 {
        return;
    }

    if let Ok(equipmemnt) = equipment_query.get(trigger.target()) {
        if let Some((box_entity, _, box_children)) = action_box_query
            .iter()
            .find(|(_, action_box, _)| action_box.slot == equipmemnt.slot)
        {
            // When on cooldown we don't want red error flash over action box
            for child in box_children.iter() {
                if error_flash_query.contains(child) {
                    commands.entity(child).despawn();
                }
            }

            commands.entity(box_entity).with_children(|parent| {
                parent.spawn((
                    CooldownIndicator,
                    Node {
                        width: Val::Px(ACTION_BOX_INTERIOR_SIZE),
                        height: Val::Px(ACTION_BOX_INTERIOR_SIZE),
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        ..default()
                    },
                    LiveDuration::new(equipmemnt.use_rate.remaining_secs()),
                    BackgroundColor::from(COOLDOWN_LINE_COLOR),
                ));
            });
        }
    }
}

pub fn on_equipment_use_failed(
    trigger: Trigger<EquipmentUseFailedEvent>,
    player: Single<(Entity, &Player)>,
    mut commands: Commands,
    action_box_query: Query<(Entity, &ActionBox)>,
) {
    if trigger.target() != player.0 {
        return;
    }

    if let Some((box_entity, _)) = action_box_query
        .iter()
        .find(|(_, action_box)| action_box.slot == trigger.slot)
    {
        if trigger.reason == EquipmentUseFailure::OutOfMana {
            commands.entity(box_entity).with_children(|parent| {
                parent.spawn((
                    ErrorFlash,
                    Node {
                        width: Val::Px(ACTION_BOX_INTERIOR_SIZE),
                        height: Val::Px(ACTION_BOX_INTERIOR_SIZE),
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        ..default()
                    },
                    BackgroundColor::from(ERROR_FLASH_COLOR),
                ));
            });
        }
    }
}

pub fn update_cooldowns(
    mut cooldown_query: Query<(&mut Node, &LiveDuration), With<CooldownIndicator>>,
) {
    for (mut line_node, cooldown_duration) in cooldown_query.iter_mut() {
        line_node.height =
            Val::Px(ACTION_BOX_INTERIOR_SIZE * cooldown_duration.0.fraction_remaining());
    }
}

pub fn get_action_bar_sprite(sprite: &Sprite) -> Sprite {
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

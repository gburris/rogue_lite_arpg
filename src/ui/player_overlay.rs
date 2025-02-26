use bevy::prelude::*;

use crate::{
    combat::attributes::{Health, Mana},
    items::{
        equipment::{EquipmentSlot, Equippable},
        inventory::Inventory,
        Item,
    },
    player::{components::Player, PlayerExperience, UseMainhandInputEvent, UseOffhandInputEvent},
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
    commands
        .spawn((
            PlayerOverlay,
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::new(Val::Px(20.0), Val::Px(20.0), Val::Px(20.0), Val::Px(20.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Top left container for health and mana bars
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|bars| {
                    create_attribute_bar(
                        bars,
                        HealthBar,
                        HealthLostBar { previous_hp: 100.0 },
                        HEALTH_COLOR,
                    );
                    create_attribute_bar(
                        bars,
                        ManaBar,
                        ManaLostBar {
                            previous_mana: 100.0,
                        },
                        MANA_COLOR,
                    );
                });

            parent.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            // Bottom container for EXP and Action bars, health pot stuff
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexEnd,
                    ..default()
                })
                .with_children(|bottom_container| {
                    bottom_container
                        .spawn(Node {
                            width: Val::Auto,
                            height: Val::Auto,
                            ..default()
                        })
                        .with_children(|exp_container| {
                            create_exp_bar(exp_container);
                        });

                    bottom_container
                        .spawn(Node {
                            width: Val::Auto,
                            height: Val::Auto,
                            ..default()
                        })
                        .with_children(|action_container| {
                            create_action_bar(action_container);
                        });
                });
        });
}

const ATTRIBUTE_BACKGROUND_COLOR: Color = Color::srgb(0.21, 0.21, 0.21);
const ATTRIBUTE_BAR_WIDTH: Val = Val::Px(400.0);

fn create_attribute_bar(
    child_builder: &mut ChildBuilder,
    marker_component: impl Component,
    change_component: impl Component,
    bar_color: Color,
) {
    child_builder
        .spawn((
            Node {
                width: ATTRIBUTE_BAR_WIDTH,
                height: Val::Px(15.0),
                ..default()
            },
            BackgroundColor::from(ATTRIBUTE_BACKGROUND_COLOR),
        ))
        .with_children(|attribute_builder| {
            attribute_builder.spawn((
                marker_component,
                Node {
                    width: ATTRIBUTE_BAR_WIDTH,
                    height: Val::Px(15.0),
                    ..default()
                },
                BackgroundColor::from(bar_color),
            ));

            attribute_builder.spawn((
                change_component,
                Node {
                    width: Val::Px(0.0),
                    height: Val::Px(15.0),
                    ..default()
                },
                BackgroundColor::from(BAR_CHANGE_COLOR),
            ));
        });
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

/* Exp Bar Code */
fn create_exp_bar(parent: &mut ChildBuilder) {
    parent
        .spawn(Node {
            width: Val::Px(400.0),
            height: Val::Px(20.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            // Add overflow visibility for debugging
            overflow: Overflow::visible(),
            ..default()
        })
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor::from(ATTRIBUTE_BACKGROUND_COLOR),
            ));

            bar.spawn((
                ExpBar,
                Node {
                    width: Val::Px(0.0),
                    height: Val::Px(20.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor::from(EXP_COLOR),
            ));

            // Hash marks
            for i in 1..10 {
                bar.spawn(Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(i as f32 * 40.0),
                    width: Val::Px(2.0),
                    height: Val::Px(20.0),
                    ..default()
                })
                .insert(BackgroundColor::from(Color::srgba(1.0, 1.0, 1.0, 0.3)));
            }
        });
}

pub fn update_exp_bar(
    player_exp: Option<Single<&PlayerExperience, (With<Player>, Changed<PlayerExperience>)>>,
    mut exp_bar: Single<&mut Node, With<ExpBar>>,
) {
    if let Some(player_exp) = player_exp {
        let exp = player_exp.into_inner();
        let progress = exp.current as f32 / exp.next_level_requirement as f32;
        exp_bar.width = Val::Px(400.0 * progress);
    }
}
/* Action Bar Code */
#[derive(Component)]
pub struct ActionBar;

#[derive(Component)]
pub struct ActionBox {
    slot: EquipmentSlot,
}

#[derive(Component)]
pub struct CooldownLine;

const ACTION_BOX_SIZE: f32 = 50.0;
const ACTION_BOX_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8); // 80% opaque black
const ACTION_BOX_OUTLINE_COLOR: Color = Color::srgba(0.8, 0.8, 0.8, 0.5); // Semi-transparent white

fn create_action_bar(parent: &mut ChildBuilder) {
    parent
        .spawn((
            ActionBar,
            Node {
                width: Val::Auto,
                height: Val::Px(ACTION_BOX_SIZE),
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(2.0)),
                ..default()
            },
        ))
        .with_children(|action_bar| {
            // Define equipment slots in order
            let slots = [
                EquipmentSlot::Mainhand,
                EquipmentSlot::Offhand,
                // TODO: Add Spell Slot 1, Spell Slot 2, etc.
            ];

            // Create boxes for equipment slots
            for slot in slots {
                action_bar
                    .spawn((
                        ActionBox { slot },
                        Node {
                            width: Val::Px(ACTION_BOX_SIZE),
                            height: Val::Px(ACTION_BOX_SIZE),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor::from(ACTION_BOX_COLOR),
                        BorderColor::from(ACTION_BOX_OUTLINE_COLOR),
                    ))
                    .with_children(|action_box| {
                        action_box.spawn((
                            ImageNode { ..default() },
                            Node {
                                width: Val::Percent(100.),
                                height: Val::Percent(100.),
                                ..default()
                            },
                        ));
                    });
            }

            // Add remaining generic action boxes
            for _ in 0..(5 - slots.len()) {
                action_bar
                    .spawn((
                        Node {
                            width: Val::Px(ACTION_BOX_SIZE),
                            height: Val::Px(ACTION_BOX_SIZE),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor::from(ACTION_BOX_COLOR),
                        BorderColor::from(ACTION_BOX_OUTLINE_COLOR),
                    ))
                    .with_children(|action_box| {
                        action_box.spawn((
                            ImageNode { ..default() },
                            Node {
                                width: Val::Percent(100.),
                                height: Val::Percent(100.),
                                ..default()
                            },
                        ));
                    });
            }
        });
}

pub fn update_action_bar(
    action_box_query: Query<(&ActionBox, &Children)>,
    mut image_query: Query<&mut ImageNode>,
    inventory_query: Option<Single<&Inventory, (Changed<Inventory>, With<Player>)>>,
    item_query: Query<(&Item, &Sprite)>,
) {
    if let Some(player_inventory_result) = inventory_query {
        let player_inventory = player_inventory_result.into_inner();

        // Update all action boxes that correspond to equipment slots
        for (action_box, children) in action_box_query.iter() {
            if let Some(equipped_entity) = player_inventory.get_equipped(action_box.slot) {
                if let Some(&image_entity) = children.first() {
                    if let Ok(mut image_node) = image_query.get_mut(image_entity) {
                        if let Ok((_, item_sprite)) = item_query.get(equipped_entity) {
                            image_node.image = item_sprite.image.clone();
                        }
                    }
                }
            }
        }
    }
}

const COOLDOWN_LINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.6);

#[derive(Component)]
pub struct CooldownIndicator {
    timer: Timer,
    slot: EquipmentSlot,
}

// Event handling for mainhand activation
pub fn on_main_hand_activated(
    trigger: Trigger<UseMainhandInputEvent>,
    mut commands: Commands,
    player_query: Single<Entity, With<Player>>,
    action_box_query: Query<(Entity, &ActionBox)>,
    inventory_query: Single<&Inventory, With<Player>>,
    weapon_query: Query<&Equippable>,
) {
    if (trigger.entity()) != player_query.into_inner() {
        return;
    }

    let player_inventory = inventory_query.into_inner();
    handle_equipment_activation(
        &mut commands,
        &action_box_query,
        player_inventory,
        &weapon_query,
        EquipmentSlot::Mainhand,
    );
}

// Event handling for offhand activation
pub fn on_off_hand_activated(
    trigger: Trigger<UseOffhandInputEvent>,
    mut commands: Commands,
    player_query: Single<Entity, With<Player>>,
    action_box_query: Query<(Entity, &ActionBox)>,
    inventory_query: Single<&Inventory, With<Player>>,
    weapon_query: Query<&Equippable>,
) {
    if (trigger.entity()) != player_query.into_inner() {
        return;
    }

    let player_inventory = inventory_query.into_inner();
    handle_equipment_activation(
        &mut commands,
        &action_box_query,
        player_inventory,
        &weapon_query,
        EquipmentSlot::Offhand,
    );
}

// Helper function to reduce code duplication in activation handlers
fn handle_equipment_activation(
    commands: &mut Commands,
    action_box_query: &Query<(Entity, &ActionBox)>,
    player_inventory: &Inventory,
    weapon_query: &Query<&Equippable>,
    slot: EquipmentSlot,
) {
    // Find the action box matching the equipment slot
    if let Some((box_entity, _)) = action_box_query
        .iter()
        .find(|(_, action_box)| action_box.slot == slot)
    {
        if let Some(weapon_entity) = player_inventory.get_equipped(slot) {
            if let Ok(weapon) = weapon_query.get(weapon_entity) {
                commands.entity(box_entity).insert(CooldownIndicator {
                    timer: weapon.use_rate.clone(),
                    slot,
                });
            }
        }
    }
}

pub fn on_cooldown_indicator_added(
    mut commands: Commands,
    query: Query<Entity, Added<CooldownIndicator>>,
) {
    for entity in query.iter() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                CooldownLine,
                Node {
                    width: Val::Percent(98.),
                    height: Val::Px(ACTION_BOX_SIZE),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor::from(COOLDOWN_LINE_COLOR),
            ));
        });
    }
}

pub fn update_cooldowns(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut CooldownIndicator, &Children)>,
    mut line_query: Query<&mut Node, With<CooldownLine>>,
) {
    for (entity, mut cooldown, children) in query.iter_mut() {
        cooldown.timer.tick(time.delta());

        if let Some(&line_entity) = children.iter().find(|&&e| line_query.contains(e)) {
            if cooldown.timer.finished() {
                // Remove immediately if finished
                commands.entity(line_entity).despawn_recursive();
                commands.entity(entity).remove::<CooldownIndicator>();
            } else if let Ok(mut line_node) = line_query.get_mut(line_entity) {
                // Otherwise update the line height
                let progress = 1.0 - cooldown.timer.fraction_remaining();
                line_node.height = Val::Px(ACTION_BOX_SIZE * (1.0 - progress));
            }
        }
    }
}

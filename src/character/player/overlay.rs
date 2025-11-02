use bevy::{ecs::spawn::SpawnIter, prelude::*};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_exp_bar,
            update_action_bar,
            update_cooldowns,
            (update_mana_bar, update_lost_mana_bar).chain(),
            (update_health_bar, update_lost_health_bar).chain(),
        )
            .in_set(InGameSystems::HudOverlay),
    );

    app.add_observer(despawn_all::<RestartEvent, PlayerOverlay>);
}

#[derive(Component)]
struct PlayerOverlay;

#[derive(Component)]
struct ManaBar;

#[derive(Component)]
struct ManaLostBar {
    previous_mana: f32,
}

#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct HealthLostBar {
    previous_hp: f32,
}

#[derive(Component, Debug)]
struct ExpBar;

const EXP_COLOR: Color = Color::srgb(0.5, 0.0, 0.5);
const HEALTH_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const MANA_COLOR: Color = Color::srgb(0.0, 0.173, 0.878);
const BAR_CHANGE_COLOR: Color = Color::srgb(1.0, 0.89, 0.41);

// If health is 100, the health bar will be 400 pixels long. Same for mana.
const ATTRIBUTE_TO_PIXEL_SCALE: f32 = 4.0;

// Represents how fast the yellow "amount lost" of health or mana goes away
const LOST_AMOUNT_SHRINK_RATE: f32 = 80.0;

pub(super) fn spawn_player_overlay(mut commands: Commands) {
    commands.spawn((
        PlayerOverlay,
        Node {
            width: percent(100.),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: px(20.0).all(),
            ..default()
        },
        children![
            // Top left container for health and mana bars
            (
                Node {
                    width: percent(100.0),
                    height: auto(),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10.0),
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
                    width: percent(100.0),
                    height: auto(),
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
            height: px(15.0),
            ..default()
        },
        BackgroundColor::from(ATTRIBUTE_BACKGROUND_COLOR),
        children![
            (
                marker_component,
                Node {
                    width: ATTRIBUTE_BAR_WIDTH,
                    height: px(15.0),
                    ..default()
                },
                BackgroundColor::from(bar_color),
            ),
            (
                change_component,
                Node {
                    width: px(0.0),
                    height: px(15.0),
                    ..default()
                },
                BackgroundColor::from(BAR_CHANGE_COLOR),
            )
        ],
    )
}

fn update_health_bar(
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

fn update_mana_bar(
    player_mana: Single<&Mana, (With<Player>, Changed<Mana>)>,
    mut mana_bar: Single<&mut Node, (With<ManaBar>, Without<ManaLostBar>)>,
    mana_lost_bar: Single<(&mut Node, &mut ManaLostBar)>,
) {
    let (mut mana_lost_node, mut mana_lost) = mana_lost_bar.into_inner();

    mana_bar.width = get_amount_left_in_pixels(player_mana.current_mana, player_mana.max_mana);
    mana_lost_node.width = get_amount_lost_in_pixels(
        mana_lost.previous_mana,
        player_mana.current_mana,
        mana_lost_node.width,
    );

    mana_lost.previous_mana = player_mana.current_mana;
}

fn update_lost_mana_bar(mut mana_lost_node: Single<&mut Node, With<ManaLostBar>>, time: Res<Time>) {
    let Val::Px(current_pixel) = mana_lost_node.width else {
        panic!("Non-pixel value for mana bar");
    };

    let amount_to_remove = LOST_AMOUNT_SHRINK_RATE * time.delta_secs();
    mana_lost_node.width = px((current_pixel - amount_to_remove).max(0.0));
}

fn update_lost_health_bar(
    mut health_lost_node: Single<&mut Node, With<HealthLostBar>>,
    time: Res<Time>,
) {
    let Val::Px(current_pixel) = health_lost_node.width else {
        panic!("Non-pixel value for mana bar");
    };

    let amount_to_remove = LOST_AMOUNT_SHRINK_RATE * time.delta_secs();
    health_lost_node.width = px((current_pixel - amount_to_remove).max(0.0));
}

// Gets length in px of bar representing amount of mana or health left
fn get_amount_left_in_pixels(current_amount: f32, max_amount: f32) -> Val {
    let max_bar_length = max_amount * ATTRIBUTE_TO_PIXEL_SCALE;
    let ratio_remaining = current_amount / max_amount;
    px(ratio_remaining * max_bar_length)
}

// Gets length in px of yellow bar representing amount of mana or health lost
fn get_amount_lost_in_pixels(previous_amount: f32, current_amount: f32, pixel_width: Val) -> Val {
    let pixel_change = (previous_amount - current_amount) * ATTRIBUTE_TO_PIXEL_SCALE;

    let Val::Px(current_pixels) = pixel_width else {
        panic!("Non-pixel value for amount lost bar");
    };

    // Negative pixel values arne't allowed
    px((current_pixels + pixel_change).max(0.0))
}

fn experience_bar() -> impl Bundle {
    (
        Node {
            width: px(400.0),
            height: px(20.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            // Add overflow visibility for debugging
            overflow: Overflow::visible(),
            ..default()
        },
        children![
            (
                Node {
                    width: px(400.0),
                    height: px(20.0),
                    ..default()
                },
                BackgroundColor::from(ATTRIBUTE_BACKGROUND_COLOR),
            ),
            (
                ExpBar,
                Node {
                    width: px(0.0),
                    height: px(20.0),
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    ..default()
                },
                BackgroundColor::from(EXP_COLOR),
                Children::spawn(SpawnIter((1..10).map(|i| (
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(i as f32 * 40.0),
                        width: px(2.0),
                        height: px(20.0),
                        ..default()
                    },
                    BackgroundColor::from(Color::srgba(1.0, 1.0, 1.0, 0.3)),
                ))))
            )
        ],
    )
}

fn update_exp_bar(
    player: Option<Single<&Player, Changed<Player>>>,
    mut exp_bar: Single<&mut Node, With<ExpBar>>,
) {
    if let Some(player) = player {
        exp_bar.width = px(400.0 * player.get_progress_to_next_level());
    }
}

#[derive(Component)]
pub(super) struct ActionBox {
    slot: EquipmentSlot,
}

#[derive(Component)]
struct CooldownIndicator;

#[derive(Component)]
#[require(Lifespan::new(0.1))]
pub(super) struct ErrorFlash;

const ACTION_BOX_SIZE: f32 = 50.0;
const ACTION_BOX_BORDER: f32 = 2.0;
const ACTION_BOX_INTERIOR_SIZE: f32 = ACTION_BOX_SIZE - (ACTION_BOX_BORDER * 2.0);
const ACTION_BOX_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8); // 80% opaque black
const ACTION_BOX_OUTLINE_COLOR: Color = Color::srgba(0.8, 0.8, 0.8, 0.5); // Semi-transparent white
const COOLDOWN_LINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.6); // Semi-transparent white
const ERROR_FLASH_COLOR: Color = Color::srgba(0.9, 0.2, 0.2, 0.2); // Semi-transparent red

fn action_bar() -> impl Bundle {
    (
        Name::new("Action Bar"),
        Node {
            flex_direction: FlexDirection::Row,
            ..default()
        },
        Children::spawn(SpawnIter(
            [EquipmentSlot::Mainhand, EquipmentSlot::Offhand]
                .iter()
                .map(|slot| action_box(*slot)),
        )),
    )
}

fn action_box(slot: EquipmentSlot) -> impl Bundle {
    (
        ActionBox { slot },
        Node {
            width: px(ACTION_BOX_SIZE),
            height: px(ACTION_BOX_SIZE),
            border: px(ACTION_BOX_BORDER).all(),
            ..default()
        },
        BackgroundColor::from(ACTION_BOX_COLOR),
        BorderColor::from(ACTION_BOX_OUTLINE_COLOR),
        Children::spawn_one((
            ImageNode::default(),
            Node {
                width: percent(100.0),
                height: percent(100.0),
                ..default()
            },
        )),
    )
}

fn update_action_bar(
    action_box_query: Query<(&ActionBox, &Children)>,
    mut image_query: Query<&mut ImageNode>,
    equipment_query: Option<
        Single<(Option<&Mainhand>, Option<&Offhand>), (Changed<Items>, With<Player>)>,
    >,
    item_query: Query<&Sprite, With<Item>>,
) {
    if let Some(player_inventory_result) = equipment_query {
        let (mainhand, offhand) = player_inventory_result.into_inner();

        for (action_box, children) in action_box_query.iter() {
            let equipment: Option<Entity> = match action_box.slot {
                EquipmentSlot::Mainhand => mainhand.map(|m| m.get()),
                EquipmentSlot::Offhand => offhand.map(|o| o.get()),
            };

            if let Some(&image_entity) = children.first()
                && let Ok(mut image_node) = image_query.get_mut(image_entity)
                && let Some(equipped_entity) = equipment
                && let Ok(item_sprite) = item_query.get(equipped_entity)
            {
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

pub(super) fn on_equipment_use_success(
    equipment_used: On<UseEquipment>,
    mut commands: Commands,
    action_box_query: Query<(Entity, &ActionBox, &Children)>,
    equipment_query: Query<&Equippable, With<Equipped>>,
    error_flash_query: Query<Entity, With<ErrorFlash>>,
) {
    if let Ok(equipmemnt) = equipment_query.get(equipment_used.entity)
        && let Some((box_entity, _, box_children)) = action_box_query
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
                    width: px(ACTION_BOX_INTERIOR_SIZE),
                    height: px(ACTION_BOX_INTERIOR_SIZE),
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    ..default()
                },
                Lifespan::new(equipmemnt.use_rate.remaining_secs()),
                BackgroundColor::from(COOLDOWN_LINE_COLOR),
            ));
        });
    }
}

pub(super) fn on_equipment_use_failed(
    equipment_use_failed: On<EquipmentUseFailed>,
    mut commands: Commands,
    action_box_query: Query<(Entity, &ActionBox)>,
) {
    if let Some((box_entity, _)) = action_box_query
        .iter()
        .find(|(_, action_box)| action_box.slot == equipment_use_failed.slot)
    {
        commands.entity(box_entity).with_child((
            ErrorFlash,
            Node {
                width: px(ACTION_BOX_INTERIOR_SIZE),
                height: px(ACTION_BOX_INTERIOR_SIZE),
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                ..default()
            },
            BackgroundColor::from(ERROR_FLASH_COLOR),
        ));
    }
}

fn update_cooldowns(mut cooldown_query: Query<(&mut Node, &Lifespan), With<CooldownIndicator>>) {
    for (mut line_node, cooldown_duration) in cooldown_query.iter_mut() {
        line_node.height = px(ACTION_BOX_INTERIOR_SIZE * cooldown_duration.0.fraction_remaining());
    }
}

fn get_action_bar_sprite(sprite: &Sprite) -> Sprite {
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

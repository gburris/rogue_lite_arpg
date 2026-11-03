use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_experience_bar,
            update_action_bar,
            update_cooldowns,
            update_mana_bar,
            update_lost_mana_bar,
            update_health_bar,
            update_lost_health_bar,
        )
            .chain()
            .in_set(InGameSystems::HudOverlay),
    );

    app.add_observer(despawn_all::<Restart, PlayerOverlay>);
}

#[derive(Component, Default, Clone)]
struct PlayerOverlay;

#[derive(Component, Default, Clone)]
struct ManaBar;

#[derive(Component, Clone)]
struct ManaLostBar {
    previous_mana: f32,
}

#[derive(Component, Default, Clone)]
struct HealthBar;

#[derive(Component, Clone)]
struct HealthLostBar {
    previous_hp: f32,
}

#[derive(Component, Debug, Default, Clone)]
struct ExperienceBar;

const EXPERIENCE_COLOR: Color = Color::srgb(0.5, 0.0, 0.5);
const HEALTH_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const MANA_COLOR: Color = Color::srgb(0.0, 0.173, 0.878);
const BAR_CHANGE_COLOR: Color = Color::srgb(1.0, 0.89, 0.41);

// If health is 100, the health bar will be 400 pixels long. Same for mana.
const ATTRIBUTE_TO_PIXEL_SCALE: f32 = 4.0;

// Represents how fast the yellow "amount lost" of health or mana goes away
const LOST_AMOUNT_SHRINK_RATE: f32 = 80.0;

pub(super) fn spawn_player_overlay(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        PlayerOverlay
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: px(20),
        }
        Children [
            // Top left container for health and mana bars
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
            }
            Children [
                @attribute_bar(HealthBar, HealthLostBar { previous_hp: 100.0 }, HEALTH_COLOR)
                --
                @attribute_bar(ManaBar, ManaLostBar { previous_mana: 100.0 }, MANA_COLOR)
            ]
            --
            @spacer()
            --
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
            }
            Children [
                @experience_bar()
                --
                @action_bar()
            ]
        ]
    });
}

const ATTRIBUTE_BACKGROUND_COLOR: Color = Color::srgb(0.21, 0.21, 0.21);
const ATTRIBUTE_BAR_WIDTH: Val = Val::Px(400.0);
const ATTRIBUTE_BAR_HEIGHT: Val = Val::Px(15.0);

/// A bar showing an attribute in `bar_color`, with a yellow tail for the amount just lost.
/// `marker` tags the filled bar and `change_marker` the tail.
fn attribute_bar(
    marker: impl Component + Clone + Unpin,
    change_marker: impl Component + Clone + Unpin,
    bar_color: Color,
) -> impl Scene {
    bsn! {
        Node {
            width: ATTRIBUTE_BAR_WIDTH,
            height: ATTRIBUTE_BAR_HEIGHT,
        }
        BackgroundColor(ATTRIBUTE_BACKGROUND_COLOR)
        Children [
            marker
            Node {
                width: ATTRIBUTE_BAR_WIDTH,
                height: ATTRIBUTE_BAR_HEIGHT,
            }
            BackgroundColor(bar_color)
            --
            change_marker
            Node {
                width: px(0),
                height: ATTRIBUTE_BAR_HEIGHT,
            }
            BackgroundColor(BAR_CHANGE_COLOR)
        ]
    }
}

fn update_health_bar(
    player_health: Option<Single<&Health, (With<Player>, Changed<Health>)>>,
    mut health_bar: Single<&mut Node, (With<HealthBar>, Without<HealthLostBar>)>,
    health_lost_bar: Single<(&mut Node, &mut HealthLostBar)>,
) {
    let (mut health_lost_node, mut health_lost) = health_lost_bar.into_inner();

    if let Some(player_health) = player_health {
        health_bar.width = player_health.fraction().map_or(px(0.0), |fraction| {
            px(fraction * player_health.max_hp * ATTRIBUTE_TO_PIXEL_SCALE)
        });
        health_lost_node.width = amount_lost_in_pixels(
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

    mana_bar.width = player_mana.fraction().map_or(px(0.0), |fraction| {
        px(fraction * player_mana.max * ATTRIBUTE_TO_PIXEL_SCALE)
    });
    mana_lost_node.width = amount_lost_in_pixels(
        mana_lost.previous_mana,
        player_mana.current,
        mana_lost_node.width,
    );

    mana_lost.previous_mana = player_mana.current;
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
        panic!("Non-pixel value for health bar");
    };

    let amount_to_remove = LOST_AMOUNT_SHRINK_RATE * time.delta_secs();
    health_lost_node.width = px((current_pixel - amount_to_remove).max(0.0));
}

// Gets length in px of yellow bar representing amount of mana or health lost
fn amount_lost_in_pixels(previous_amount: f32, current_amount: f32, pixel_width: Val) -> Val {
    let pixel_change = (previous_amount - current_amount) * ATTRIBUTE_TO_PIXEL_SCALE;

    let Val::Px(current_pixels) = pixel_width else {
        panic!("Non-pixel value for amount lost bar");
    };

    // Negative pixel values arne't allowed
    px((current_pixels + pixel_change).max(0.0))
}

fn experience_bar() -> impl Scene {
    // Tick marks splitting the bar into tenths.
    let ticks: Vec<_> = (1..10)
        .map(|i| {
            bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    left: px({i as f32 * 40.0}),
                    width: px(2),
                    height: px(20),
                }
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.3))
            }
        })
        .collect();

    bsn! {
        Node {
            width: px(400),
            height: px(20),
        }
        BackgroundColor(ATTRIBUTE_BACKGROUND_COLOR)
        Children [
            ExperienceBar
            Node {
                width: px(0),
                height: px(20),
                position_type: PositionType::Absolute,
                left: px(0),
            }
            BackgroundColor(EXPERIENCE_COLOR)
            Children [
                {ticks}
            ]
        ]
    }
}

fn update_experience_bar(
    player: Option<Single<&Player, Changed<Player>>>,
    mut experience_bar: Single<&mut Node, With<ExperienceBar>>,
) {
    if let Some(player) = player {
        experience_bar.width = px(400.0 * player.progress_to_next_level());
    }
}

#[derive(Component, Clone)]
pub(super) struct ActionBox {
    slot: EquipmentSlot,
}

#[derive(Component, Default, Clone)]
struct CooldownIndicator;

#[derive(Component, Default, Clone)]
#[require(Lifespan::new(0.1))]
pub(super) struct ErrorFlash;

const ACTION_BOX_SIZE: f32 = 50.0;
const ACTION_BOX_BORDER: f32 = 2.0;
const ACTION_BOX_INTERIOR_SIZE: f32 = ACTION_BOX_SIZE - (ACTION_BOX_BORDER * 2.0);
const ACTION_BOX_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8); // 80% opaque black
const ACTION_BOX_OUTLINE_COLOR: Color = Color::srgba(0.8, 0.8, 0.8, 0.5); // Semi-transparent white
const COOLDOWN_LINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.6); // Semi-transparent white
const ERROR_FLASH_COLOR: Color = Color::srgba(0.9, 0.2, 0.2, 0.2); // Semi-transparent red

fn action_bar() -> impl Scene {
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
        ~{ActionBox { slot }}
        Node {
            width: px(ACTION_BOX_SIZE),
            height: px(ACTION_BOX_SIZE),
            border: px(ACTION_BOX_BORDER),
        }
        BackgroundColor(ACTION_BOX_COLOR)
        BorderColor::all(ACTION_BOX_OUTLINE_COLOR)
        Children [
            ImageNode
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

fn update_action_bar(
    action_box_query: Query<(&ActionBox, &Children)>,
    mut image_query: Query<&mut ImageNode>,
    equipment_query: Option<
        Single<
            (Option<&Mainhand>, Option<&Offhand>),
            (Or<(Changed<Mainhand>, Changed<Offhand>)>, With<Player>),
        >,
    >,
    item_query: Query<&Sprite, With<Item>>,
) {
    if let Some(player_inventory_result) = equipment_query {
        let (mainhand, offhand) = player_inventory_result.into_inner();

        for (action_box, children) in &action_box_query {
            let equipment: Option<Entity> = action_box.slot.equipped(mainhand, offhand);

            if let Some(&image_entity) = children.first()
                && let Ok(mut image_node) = image_query.get_mut(image_entity)
                && let Some(equipped_entity) = equipment
                && let Ok(item_sprite) = item_query.get(equipped_entity)
            {
                let action_bar_sprite = action_bar_sprite(item_sprite);

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

/// Adds a cooldown indicator to the action box for the used slot.
pub(super) fn on_equipment_use_succeeded(
    equipment_used: On<EquipmentUseSucceeded>,
    mut commands: Commands,
    action_box_query: Query<(Entity, &ActionBox)>,
    equipment_query: Query<&Equippable, With<Equipped>>,
) {
    if let Ok(equippable) = equipment_query.get(equipment_used.item)
        && let Some((box_entity, _)) = action_box_query
            .iter()
            .find(|(_, action_box)| action_box.slot == equipment_used.slot)
    {
        let cooldown = equippable.cooldown.remaining_secs();
        commands.spawn_scene(bsn! {
            @action_box_overlay(box_entity, COOLDOWN_LINE_COLOR)
            CooldownIndicator
            Lifespan::new(cooldown)
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
        && equipment_use_failed.reason != EquipmentUseFailure::OnCooldown
    {
        commands.spawn_scene(bsn! {
            @action_box_overlay(box_entity, ERROR_FLASH_COLOR)
            ErrorFlash
        });
    }
}

fn update_cooldowns(mut cooldown_query: Query<(&mut Node, &Lifespan), With<CooldownIndicator>>) {
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

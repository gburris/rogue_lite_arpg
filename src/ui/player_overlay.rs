use bevy::prelude::*;

use crate::{
    combat::attributes::{Health, Mana},
    player::{components::Player, PlayerExperience, PlayerLevel},
};

#[derive(Component)]
pub struct GameOverlay;

#[derive(Component)]
pub struct PlayerOverlayStatsText;

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

// Add a new component for the exp bar
#[derive(Component)]
pub struct ExpBar;

// Add this new color constant
const EXP_COLOR: Color = Color::srgb(0.5, 0.0, 0.5); // Purple color for exp bar

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
            GameOverlay,
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

            // Spacer
            parent.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            // Bottom right exp bar container
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    justify_content: JustifyContent::FlexStart, // Ensures children align to the left
                    align_items: AlignItems::FlexStart,         // Aligns items to the left
                    ..default()
                })
                .with_children(|exp_container| {
                    create_exp_bar(exp_container);
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

fn create_exp_bar(parent: &mut ChildBuilder) {
    parent
        .spawn(Node {
            width: Val::Px(400.0),
            height: Val::Px(20.0),
            ..default()
        })
        .with_children(|bar| {
            // Background
            bar.spawn((
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor::from(ATTRIBUTE_BACKGROUND_COLOR),
            ));

            // Fill bar
            bar.spawn((
                ExpBar,
                Node {
                    width: Val::Px(0.0),
                    height: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor::from(EXP_COLOR),
            ));

            // Hash marks
            for i in 1..10 {
                bar.spawn(Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(i as f32 * 40.0), // 400px / 10 sections = 40px per section
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
        warn!("updating exp bar");
        warn!("Exp current {}", exp.current);
        warn!("Exp needed {}", exp.next_level_requirement);
        let progress = exp.current as f32 / exp.next_level_requirement as f32;
        warn!("Exp progress {}", progress);
        exp_bar.width = Val::Px(400.0 * progress);
    }
}

use bevy::prelude::*;

use super::action_bar::{self, action_bar};
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_experience_bar,
            update_mana_bar,
            shrink_lost_bar::<ManaLostBar>,
            update_health_bar,
            shrink_lost_bar::<HealthLostBar>,
        )
            .chain()
            .in_set(InGameSystems::HudOverlay)
            // Cooldown indicators and the bars are different UI nodes.
            .ambiguous_with(action_bar::update_cooldowns),
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

fn shrink_lost_bar<LostBar: Component>(
    mut lost_bar_node: Single<&mut Node, With<LostBar>>,
    time: Res<Time>,
) {
    let Val::Px(current_pixels) = lost_bar_node.width else {
        panic!("Non-pixel value for amount lost bar");
    };

    let amount_to_remove = LOST_AMOUNT_SHRINK_RATE * time.delta_secs();
    lost_bar_node.width = px((current_pixels - amount_to_remove).max(0.0));
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

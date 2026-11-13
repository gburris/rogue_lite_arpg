use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_change_stat)
        .add_observer(on_update_stats_ui)
        .add_systems(OnEnter(Menu::StatsShop), spawn_stats_shop_menu);
}

#[derive(Component, Default, Clone)]
pub struct StatShopMenu;

/// Which way a stats shop button moves its attribute.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatChange {
    #[default]
    Increase,
    Decrease,
}

#[derive(Component, Clone, Default)]
pub struct StatShopButton {
    pub attribute: Attribute,
    pub change: StatChange,
}

#[derive(Event)]
pub struct ChangeStat {
    pub attribute: Attribute,
    pub change: StatChange,
}

#[derive(Event)]
pub struct UpdateStatsUi;

pub fn spawn_stats_shop_menu(
    mut commands: Commands,
    player_attributes: Single<&PlayerAttributes>,
    game_progress: ResMut<GameProgress>,
) {
    let attributes = player_attributes.into_inner();
    let points = game_progress.progress_points;
    let rows: Vec<_> = Attribute::ALL
        .into_iter()
        .map(|attribute| attribute_row(attribute, attributes, points))
        .collect();

    commands.spawn_scene(bsn! {
        @menu_screen(Menu::StatsShop, "STATS SHOP")
        StatShopMenu
        Children [
            @panel(600.0)
            Children [
                {rows}
            ]
            --
            @text({format!("Available Progress Points: {points}")}, 32.0)
        ]
    });
}

fn attribute_row(attribute: Attribute, attributes: &PlayerAttributes, points: u32) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: px(50),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: {px(10).horizontal()},
        }
        Children [
            @attribute_shop_button(attribute, StatChange::Decrease, points)
            --
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
            }
            Children [
                @text({format!("{attribute:?}: {}", attribute.value(attributes))}, BODY_FONT_SIZE)
                --
                @muted_text(attribute.description(), CAPTION_FONT_SIZE)
            ]
            --
            @attribute_shop_button(attribute, StatChange::Increase, points)
        ]
    }
}

fn attribute_shop_button(attribute: Attribute, change: StatChange, points: u32) -> impl Scene {
    let (label, highlight) = match change {
        // Highlight increases the player can afford.
        StatChange::Increase if points > 0 => ("+", Color::srgb(1.0, 1.0, 0.0)),
        StatChange::Increase => ("+", Color::srgb(0.5, 0.5, 0.5)),
        StatChange::Decrease => ("-", Color::srgb(0.5, 0.5, 0.5)),
    };
    let colors = ButtonColors {
        idle: ROW_COLOR,
        hovered: highlight,
        pressed: highlight,
    };

    bsn! {
        @button(label, BODY_FONT_SIZE, Color::WHITE, colors)
        StatShopButton { attribute, change }
        Node {
            width: px(30),
            height: px(30),
        }
        on(on_stat_button_activated)
    }
}

fn on_stat_button_activated(
    activate: On<Activate>,
    buttons: Query<&StatShopButton>,
    mut commands: Commands,
    game_progress: Res<GameProgress>,
    player_attributes: Single<&PlayerAttributes>,
) {
    let Ok(button) = buttons.get(activate.entity) else {
        return;
    };

    let can_increase = button.change == StatChange::Increase && game_progress.progress_points > 0;
    let can_decrease =
        button.change == StatChange::Decrease && button.attribute.value(*player_attributes) > 1;

    if can_increase || can_decrease {
        commands.trigger(ChangeStat {
            attribute: button.attribute,
            change: button.change,
        });
    }
}

pub fn on_change_stat(
    trigger: On<ChangeStat>,
    mut player_attributes: Query<&mut PlayerAttributes>,
    mut game_progress: ResMut<GameProgress>,
    mut commands: Commands,
) {
    let mut attributes = player_attributes
        .single_mut()
        .expect("player has no PlayerAttributes");

    match (trigger.attribute, trigger.change) {
        (attribute, StatChange::Increase) if game_progress.progress_points > 0 => {
            match attribute {
                Attribute::Agility => attributes.agility += 1,
                Attribute::Strength => attributes.strength += 1,
                Attribute::Dexterity => attributes.dexterity += 1,
                Attribute::Intellect => attributes.intellect += 1,
                Attribute::Luck => attributes.luck += 1,
            }
            game_progress.progress_points -= 1;
            commands.trigger(UpdateStatsUi);
        }
        (attribute, StatChange::Decrease) => {
            let current_value = attribute.value(&attributes);
            if current_value > 1 {
                match attribute {
                    Attribute::Agility => attributes.agility -= 1,
                    Attribute::Strength => attributes.strength -= 1,
                    Attribute::Dexterity => attributes.dexterity -= 1,
                    Attribute::Intellect => attributes.intellect -= 1,
                    Attribute::Luck => attributes.luck -= 1,
                }
                game_progress.progress_points += 1;
                commands.trigger(UpdateStatsUi);
            }
        }
        (_, StatChange::Increase) => {}
    }
}

pub fn on_update_stats_ui(
    _: On<UpdateStatsUi>,
    mut commands: Commands,
    stats_menu_query: Query<Entity, With<StatShopMenu>>,
    player_attributes_query: Single<&PlayerAttributes>,
    mut game_progress: ResMut<GameProgress>,
) {
    // Set Game Progress to current player attributes
    let player_attributes = **player_attributes_query;
    game_progress.base_attributes = player_attributes;

    // Despawn existing menu
    for entity in &stats_menu_query {
        commands.entity(entity).despawn();
    }

    // Respawn with updated values
    spawn_stats_shop_menu(commands, player_attributes_query, game_progress);
}

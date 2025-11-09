use bevy::prelude::*;

use crate::{
    prelude::{Menu, PlayerStats},
    ui::{
        constants::color,
        element::node,
        primitives::{menu_header, text},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Stats), spawn_stats_menu);
}

#[derive(Component)]
struct StatsMenu;

#[derive(Component)]
struct StatsDisplay;

fn spawn_stats_menu(mut commands: Commands, player_stats: Query<&PlayerStats>) {
    if let Ok(stats) = player_stats.single() {
        commands.spawn((
            StatsMenu,
            DespawnOnExit(Menu::Stats),
            GlobalZIndex(2),
            node()
                .width(percent(100.0))
                .height(percent(100.0))
                .align_items(AlignItems::Center)
                .flex_direction(FlexDirection::Column)
                .row_gap(px(20.0))
                .build(),
            children![
                menu_header("STATS"),
                (
                    StatsDisplay,
                    node()
                        .width(px(600.0))
                        .height(percent(80.0))
                        .flex_direction(FlexDirection::Column)
                        .padding(px(20.0).all())
                        .build(),
                    BackgroundColor::from(color::DARK_GRAY_ALPHA),
                    children![
                        stat_row("Agility", stats.agility, "Movement speed, roll range"),
                        stat_row("Strength", stats.strength, "Melee swing damage"),
                        stat_row("Dexterity", stats.dexterity, "Critical Strike Chance"),
                        stat_row("Intellect", stats.intellect, "Spell damage"),
                        stat_row("Luck", stats.luck, "Drop rate"),
                    ],
                )
            ],
        ));
    }
}

fn stat_row(
    stat_name: impl Into<String>,
    stat_value: u32,
    description: impl Into<String>,
) -> impl Bundle {
    (
        node()
            .width(percent(100.0))
            .height(px(60.0))
            .padding(px(10.0).all())
            .margin(px(5.0).bottom())
            .justify_content(JustifyContent::SpaceBetween)
            .align_items(AlignItems::Center)
            .build(),
        BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.5)),
        children![
            // left side
            (
                node()
                    .flex_direction(FlexDirection::Column)
                    .justify_content(JustifyContent::Center)
                    .build(),
                children![
                    text(stat_name, 24.0),
                    (
                        text(description, 16.0),
                        node()
                            .margin(px(4.0).top())
                            .build(),
                    )
                ]
            ),
            // right side
            text(format!("{stat_value}/99"), 24.0)
        ],
    )
}

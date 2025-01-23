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
            parent.spawn((
                PlayerOverlayStatsText,
                Node {
                    height: Val::Px(80.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Text::new("(0.0, 0.0)"),
            ));

            // Spacer between stats text and mana bar
            parent.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                ..default()
            });

            // Footer
            parent
                .spawn((Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    ..default()
                },))
                .with_children(|footer| {
                    footer
                        .spawn((
                            Node {
                                width: Val::Px(400.0),
                                height: Val::Px(15.0),
                                ..default()
                            },
                            BackgroundColor::from(Color::srgb(0.21, 0.21, 0.21)),
                        ))
                        .with_child((
                            ManaBar,
                            Node {
                                width: Val::Px(400.0),
                                height: Val::Px(15.0),
                                ..default()
                            },
                            BackgroundColor::from(Color::srgb(0.0, 0.173, 0.878)),
                        ));
                });
        });
}

pub fn update(
    player: Single<(&PlayerExperience, &PlayerLevel, &Health, &Mana), With<Player>>,
    overlay_stat_text: Single<&mut Text, With<PlayerOverlayStatsText>>,
) {
    let (exp, level, health, mana) = player.into_inner();

    let mut overlay_stat_text = overlay_stat_text.into_inner();
    *overlay_stat_text = Text::new(format!(
        "Level: {:.1} Exp: {:.1} / {:.1} ||| Health: {:.1} / {:.1} ||| Mana: {:.1} / {:.1}",
        level.current,
        exp.current,
        exp.next_level_requirement,
        health.hp,
        health.max_hp,
        mana.current_mana,
        mana.max_mana
    ));
}

pub fn update_mana_bar(
    player_mana: Single<&Mana, (With<Player>, Changed<Mana>)>,
    mana_bar: Single<&mut Node, With<ManaBar>>,
) {
    let mana = player_mana.into_inner();
    let mut mana_bar = mana_bar.into_inner();

    // Mana bar gets longer as player gets higher max mana
    let max_mana_bar_length = mana.max_mana * 4.0;

    mana_bar.width = Val::Px((mana.current_mana / mana.max_mana) * max_mana_bar_length);
}

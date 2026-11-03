use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Stats), spawn_stats_menu);
}

#[derive(Component, Default, Clone)]
struct StatsMenu;

fn spawn_stats_menu(mut commands: Commands, player_stats: Single<&PlayerStats>) {
    let rows: Vec<_> = Stat::ALL
        .into_iter()
        .map(|stat| stat_row(stat, *player_stats))
        .collect();

    commands.spawn_scene(bsn! {
        @menu_screen(Menu::Stats, "STATS")
        StatsMenu
        Children [
            @panel(600.0)
            Node { height: percent(80) }
            Children [
                {rows}
            ]
        ]
    });
}

fn stat_row(stat: Stat, stats: &PlayerStats) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: px(60),
            padding: px(10),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
        }
        BackgroundColor(ROW_COLOR)
        Children [
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                row_gap: px(4),
            }
            Children [
                @text({format!("{stat:?}")}, BODY_FONT_SIZE)
                --
                @muted_text(stat.description(), CAPTION_FONT_SIZE)
            ]
            --
            @text({format!("{}/99", stat.value(stats))}, BODY_FONT_SIZE)
        ]
    }
}

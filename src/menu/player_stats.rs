use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Stats), spawn_stats_menu);
}

#[derive(Component, Default, Clone)]
struct StatsMenu;

fn spawn_stats_menu(mut commands: Commands, player_attributes: Single<&PlayerAttributes>) {
    let rows: Vec<_> = Attribute::ALL
        .into_iter()
        .map(|attribute| attribute_row(attribute, *player_attributes))
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

fn attribute_row(attribute: Attribute, attributes: &PlayerAttributes) -> impl Scene {
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
                @text({format!("{attribute:?}")}, BODY_FONT_SIZE)
                --
                @muted_text(attribute.description(), CAPTION_FONT_SIZE)
            ]
            --
            @text({format!("{}/99", attribute.value(attributes))}, BODY_FONT_SIZE)
        ]
    }
}

use bevy::prelude::*;

use crate::{items::equipment::EquipmentSlot, ui::pause_menu::button_interactions::*};

#[derive(Component)]
pub struct EquipmentMenu;

#[derive(Component)]
pub struct EquipmentItemText;

#[derive(Component)]
pub struct EquipmentButton {
    pub item_entity: Option<Entity>,
}
#[derive(Event)]
pub struct UpdateEquipmentUIEvent;

pub fn spawn_equipment_slot(
    builder: &mut ChildBuilder,
    slot_type: EquipmentSlot,
    item_name: &str,
    item_entity: Option<Entity>,
) {
    let slot_name = match slot_type {
        EquipmentSlot::Mainhand => "Mainhand",
        EquipmentSlot::Helmet => "Helmet",
    };

    builder
        .spawn((
            EquipmentButton { item_entity },
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::bottom(Val::Px(5.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.5)),
        ))
        .observe(on_item_clicked)
        .observe(on_item_hover)
        .with_children(|parent| {
            // Slot name
            parent.spawn((
                Text::new(slot_name),
                TextColor::default(), // Add this line
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                Node::default(),
            ));
            if let Some(_) = item_entity {
                parent.spawn((
                    EquipmentItemText,
                    Text::new(item_name),
                    TextColor::default(), // Add this line
                    Button,
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    Node {
                        margin: UiRect::left(Val::Px(10.0)),
                        ..default()
                    },
                ));
            } else {
                parent.spawn((
                    Text::new("Empty"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    Node {
                        margin: UiRect::left(Val::Px(10.0)),
                        ..default()
                    },
                ));
            }
        });
}

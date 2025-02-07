use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    items::{
        equipment::{EquipmentSlot, EquipmentSlots},
        Item,
    },
    npc::NPC,
    player::Player,
    ui::pause_menu::button_interactions::*,
};

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

//Called after dispatching a click event
pub fn handle_equipment_update(
    _: Trigger<UpdateEquipmentUIEvent>,
    mut item_text_query: Query<(&mut Text, &Parent), With<EquipmentItemText>>,
    item_query: Query<&Name, With<Item>>,
    equpment_slots: Single<&EquipmentSlots, (With<Player>, Without<Enemy>, Without<NPC>)>,
) {
    if let Some(mainhand) = equpment_slots.mainhand {
        let name = item_query.get(mainhand).unwrap();
    }

    // Iterate through all ItemText components and fetch the parent InventorySlot component
    for (mut item_text, item_slot) in item_text_query.iter_mut() {
        if let Ok(mut item_slot) = inventory_slot_query.get_mut(item_slot.get()) {
            // If there is an item in the player inventory at this index, set item text and item slot
            if let Some(&item_entity) = player_inventory.items.get(item_slot.index) {
                item_slot.item = Some(item_entity);
                item_text.0 = item_query.get(item_entity).unwrap().to_string();
            } else {
                // Otherwise there is no item, set default stuff
                item_text.0 = "Empty Slot".to_string();
                item_slot.item = None;
            }
        }
    }
}

// pub fn handle_equipment_update(
//     _: Trigger<EquipmentUIUpdatedEvent>,
//     mut commands: Commands,
//     eqipment_menu_query: Query<Entity, With<EquipmentMenu>>,
//     item_query: Query<&Name, With<Item>>,
//     player_equipment_slots: Single<&EquipmentSlots, With<Player>>,
// ) {
//     // Despawn the existing inventory menu
//     for entity in eqipment_menu_query.iter() {
//         commands.entity(entity).despawn_recursive();
//     }
//     // Respawn the equipment menu
//     spawn_equipment_menu(commands, item_query, player_equipment_slots);
// }

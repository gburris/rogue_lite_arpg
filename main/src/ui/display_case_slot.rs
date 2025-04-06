use bevy::{ecs::spawn::SpawnWith, prelude::*};

use crate::{
    configuration::assets::GameIcons,
    items::{
        equipment::{EquipmentSlot, Equippable, Equipped},
        inventory::Inventory,
        Consumable, Item, ItemDropEvent, ItemType,
    },
    player::{systems::ConsumeEvent, Player},
};

use super::{
    display_case::{UpdateDisplayCaseEvent, EQUIP_SLOT_WIDTH, VALUE_WIDTH},
    primitives::{text, width},
};

const HOVER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);

#[derive(Component)]
pub struct DisplayCaseSlot {
    /// Index in the display case correspoding to index in actual entities inventory
    pub index: usize,
}

/// Makes building display slots easier
pub struct DisplaySlotContext<'a> {
    pub index: usize,
    pub item_name: &'a str,
    pub item: &'a Item,
    pub equipment_slot: Option<EquipmentSlot>,
    pub is_equipped: bool,
}

/// Spawns a given "slot" in a display case representing a single item in the inventory
pub fn spawn_slot(
    builder: &mut ChildSpawnerCommands,
    icons: &GameIcons,
    context: DisplaySlotContext,
) {
    let equip_slot_string = context
        .equipment_slot
        .map(|slot| slot.to_string())
        .unwrap_or("-".to_string());

    let equipped_icon = icons.equip_icon.clone();
    let is_equipped = context.is_equipped;

    builder
        .spawn((
            DisplayCaseSlot {
                index: context.index,
            },
            Node {
                width: Val::Px(900.0),
                height: Val::Px(32.0),
                padding: UiRect::all(Val::Px(5.0)),
                column_gap: Val::Px(5.0),
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable {
                should_block_lower: false,
                ..default()
            },
            Children::spawn((
                Spawn((
                    ImageNode {
                        image: match context.item.item_type {
                            ItemType::Melee => icons.melee_icon.clone(),
                            ItemType::Staff => icons.staff_icon.clone(),
                            ItemType::Potion => icons.potion_icon.clone(),
                            ItemType::Tome => icons.spell_book_icon.clone(),
                        },
                        ..default()
                    },
                    Node {
                        width: Val::Px(30.0),
                        height: Val::Px(30.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                )),
                Spawn((text(context.item_name, 18.0), Pickable::IGNORE)),
                SpawnWith(move |parent: &mut ChildSpawner| {
                    if is_equipped {
                        spawn_equip_icon(parent, equipped_icon);
                    }
                }),
                Spawn((
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                    Pickable::IGNORE,
                )),
                Spawn((
                    text(equip_slot_string, 18.0),
                    width(EQUIP_SLOT_WIDTH),
                    Pickable::IGNORE,
                )),
                Spawn((
                    text(context.item.value.to_string(), 18.0),
                    width(VALUE_WIDTH),
                    Pickable::IGNORE,
                )),
            )),
        ))
        .observe(on_slot_clicked)
        .observe(on_slot_hover)
        .observe(on_slot_done_hovering);
}

fn spawn_equip_icon(parent: &mut ChildSpawner, equip_icon: Handle<Image>) {
    parent.spawn((
        ImageNode {
            image: equip_icon,
            ..default()
        },
        Node {
            height: Val::Px(16.0),
            width: Val::Px(16.0),
            ..default()
        },
        Pickable::IGNORE,
    ));
}

pub fn on_slot_clicked(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    slot_query: Query<&DisplayCaseSlot>,
    item_query: Query<(Has<Equippable>, Has<Equipped>, Has<Consumable>), With<Item>>,
    player: Single<(Entity, &Inventory), With<Player>>,
) {
    let item_slot = slot_query.get(trigger.target()).unwrap();
    let (player_entity, inventory) = player.into_inner();
    let item_entity = inventory.items[item_slot.index];

    if let Ok((equippable, is_equipped, consumable)) = item_query.get(item_entity) {
        // Left click consumes or equips item
        if trigger.event.button == PointerButton::Primary {
            if equippable {
                if is_equipped {
                    commands.entity(item_entity).remove::<Equipped>();
                } else {
                    commands
                        .entity(item_entity)
                        .insert(Equipped::new(player_entity));
                }
            } else if consumable {
                commands.trigger_targets(ConsumeEvent { item_entity }, player_entity);
            }

        // Right click drops items from inventory
        } else if trigger.event.button == PointerButton::Secondary {
            commands.trigger_targets(ItemDropEvent, item_entity);
        }

        commands.trigger_targets(UpdateDisplayCaseEvent, player_entity);
    }
}

pub fn on_slot_hover(
    trigger: Trigger<Pointer<Over>>,
    mut item_slot: Query<&mut BackgroundColor, With<DisplayCaseSlot>>,
) {
    if let Ok(mut background_color) = item_slot.get_mut(trigger.target()) {
        background_color.0 = HOVER_COLOR;
    }
}

pub fn on_slot_done_hovering(
    trigger: Trigger<Pointer<Out>>,
    mut item_slot: Query<&mut BackgroundColor, With<DisplayCaseSlot>>,
) {
    if let Ok(mut background_color) = item_slot.get_mut(trigger.target()) {
        background_color.0 = Color::NONE;
    }
}

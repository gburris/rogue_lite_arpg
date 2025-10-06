use bevy::{ecs::spawn::SpawnWith, prelude::*, ui_widgets::observe};

use crate::{
    configuration::assets::GameIcons,
    items::{
        Consumable, ConsumeEvent, Item, ItemType,
        equipment::{EquipmentSlot, Equippable, Equipped},
        lootable::ItemDropEvent,
    },
    prelude::Player,
};

use super::{
    display_case::{EQUIP_SLOT_WIDTH, UpdateDisplayCaseEvent, VALUE_WIDTH},
    primitives::{text, width},
};

const HOVER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);

#[derive(Component)]
#[relationship(relationship_target = ItemDisplayed)]
pub struct DisplaySlotOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = DisplaySlotOf, linked_spawn)]
pub struct ItemDisplayed(Entity);

/// Makes building display slots easier
pub struct DisplaySlotContext {
    pub item_entity: Entity,
    pub item_name: String,   // Owned string
    pub item_type: ItemType, // Just store what you need
    pub item_value: u32,     // Just store what you need
    pub equipment_slot: Option<EquipmentSlot>,
    pub is_equipped: bool,
}

/// Spawns a given "slot" in a display case representing a single item in the inventory
pub fn display_slot(icons: &GameIcons, context: DisplaySlotContext) -> impl Bundle {
    let equip_slot_string = context
        .equipment_slot
        .map(|slot| slot.to_string())
        .unwrap_or("-".to_string());

    let equipped_icon = icons.equip_icon.clone();
    let is_equipped = context.is_equipped;

    (
        DisplaySlotOf(context.item_entity),
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
                    image: match context.item_type {
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
                text(context.item_value.to_string(), 18.0),
                width(VALUE_WIDTH),
                Pickable::IGNORE,
            )),
        )),
        observe(on_slot_clicked),
        observe(on_slot_hover),
        observe(on_slot_done_hovering),
    )
}

fn spawn_equip_icon(parent: &mut ChildSpawner, equipped_icon: Handle<Image>) {
    parent.spawn((
        ImageNode {
            image: equipped_icon,
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
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    slot_query: Query<&DisplaySlotOf>,
    item_query: Query<(Has<Equippable>, Has<Equipped>, Has<Consumable>), With<Item>>,
    player: Single<Entity, With<Player>>,
) {
    let item_entity = slot_query.get(trigger.event().entity).unwrap().0;

    if let Ok((equippable, is_equipped, consumable)) = item_query.get(item_entity) {
        // Left click consumes or equips item
        if trigger.event().button == PointerButton::Primary {
            if equippable {
                if is_equipped {
                    commands.entity(item_entity).remove::<Equipped>();
                } else {
                    commands.entity(item_entity).insert(Equipped);
                }
            } else if consumable {
                commands.trigger(ConsumeEvent {
                    entity: *player,
                    item_entity,
                });
            }

        // Right click drops items from inventory
        } else if trigger.event().button == PointerButton::Secondary {
            commands.trigger(ItemDropEvent {
                entity: item_entity,
            });
        }

        commands.trigger(UpdateDisplayCaseEvent { entity: *player });
    }
}

pub fn on_slot_hover(
    trigger: On<Pointer<Over>>,
    mut item_slot: Query<&mut BackgroundColor, With<DisplaySlotOf>>,
) {
    if let Ok(mut background_color) = item_slot.get_mut(trigger.target()) {
        background_color.0 = HOVER_COLOR;
    }
}

pub fn on_slot_done_hovering(
    trigger: On<Pointer<Out>>,
    mut item_slot: Query<&mut BackgroundColor, With<DisplaySlotOf>>,
) {
    if let Ok(mut background_color) = item_slot.get_mut(trigger.target()) {
        background_color.0 = Color::NONE;
    }
}

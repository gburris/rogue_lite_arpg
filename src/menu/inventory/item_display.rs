use bevy::prelude::*;

use super::display_case::UpdateDisplayCase;
use crate::prelude::*;

const HOVER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);
pub const VALUE_WIDTH: f32 = 60.0;
pub const EQUIP_SLOT_WIDTH: f32 = 150.0;

#[derive(Component, Clone)]
#[relationship(relationship_target = ItemDisplayed)]
pub struct DisplaySlotOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = DisplaySlotOf, linked_spawn)]
pub struct ItemDisplayed(Entity);

/// Makes building display slots easier.
pub struct DisplaySlotContext {
    pub item: Entity,
    pub name: String,   // Owned string
    pub kind: ItemKind, // Just store what you need
    pub value: u32,     // Just store what you need
    pub equipment_slot: Option<EquipmentSlot>,
    pub equipped: bool,
}

/// A single "slot" in a display case, representing one item in the inventory.
pub fn display_slot(icons: &GameIcons, context: DisplaySlotContext) -> impl Scene {
    let equip_slot = context
        .equipment_slot
        .map_or("-".to_string(), |slot| slot.to_string());
    let icon = match context.kind {
        ItemKind::Melee => icons.melee.clone(),
        ItemKind::Staff => icons.staff.clone(),
        ItemKind::Potion => icons.potion.clone(),
        ItemKind::Tome => icons.spell_book.clone(),
    };
    let equipped_icon = context.equipped.then(|| {
        let image = icons.equip.clone();
        bsn_list! {
            ImageNode { image }
            Node {
                width: px(16),
                height: px(16),
            }
            Pickable::IGNORE
        }
    });
    let item = context.item;

    bsn! {
        ~{DisplaySlotOf(item)}
        Node {
            width: px(900),
            height: px(32),
            padding: px(5),
            column_gap: px(5),
            align_items: AlignItems::Center,
        }
        Pickable { should_block_lower: false }
        on(on_slot_clicked)
        on(on_slot_hover)
        on(on_slot_done_hovering)
        Children [
            ImageNode { image: icon }
            Node {
                width: px(30),
                height: px(30),
            }
            Pickable::IGNORE
            --
            @text(context.name, SMALL_FONT_SIZE)
            Pickable::IGNORE
            --
            {equipped_icon}
            --
            @spacer()
            Pickable::IGNORE
            --
            @text(equip_slot, SMALL_FONT_SIZE)
            @width(EQUIP_SLOT_WIDTH)
            Pickable::IGNORE
            --
            @text({context.value.to_string()}, SMALL_FONT_SIZE)
            @width(VALUE_WIDTH)
            Pickable::IGNORE
        ]
    }
}

pub fn on_slot_clicked(
    pointer_click: On<PointerClick>,
    mut commands: Commands,
    slot_query: Query<&DisplaySlotOf>,
    item_query: Query<(Has<Equippable>, Has<Equipped>, Has<Consumable>), With<Item>>,
    player: Single<Entity, With<Player>>,
) -> Result {
    let item_entity = slot_query.get(pointer_click.entity)?.0;

    if let Ok((equippable, equipped, consumable)) = item_query.get(item_entity) {
        // Left click consumes or equips item
        if pointer_click.button == PointerButton::Primary {
            if equippable {
                if equipped {
                    commands.trigger(Unequip { item: item_entity });
                } else {
                    commands.trigger(Equip {
                        item: item_entity,
                        holder: *player,
                    });
                }
            } else if consumable {
                commands.trigger(Consume {
                    entity: *player,
                    item: item_entity,
                });
            }

        // Right click drops items from inventory
        } else if pointer_click.button == PointerButton::Secondary {
            commands.trigger(DropItem { item: item_entity });
        }

        commands.trigger(UpdateDisplayCase { entity: *player });
    }
    Ok(())
}

pub fn on_slot_hover(
    pointer_over: On<PointerOver>,
    mut item_slot: Query<&mut BackgroundColor, With<DisplaySlotOf>>,
) {
    if let Ok(mut background_color) = item_slot.get_mut(pointer_over.entity) {
        background_color.0 = HOVER_COLOR;
    }
}

pub fn on_slot_done_hovering(
    pointer_out: On<PointerOut>,
    mut item_slot: Query<&mut BackgroundColor, With<DisplaySlotOf>>,
) {
    if let Ok(mut background_color) = item_slot.get_mut(pointer_out.entity) {
        background_color.0 = Color::NONE;
    }
}

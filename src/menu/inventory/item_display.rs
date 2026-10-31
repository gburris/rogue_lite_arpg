use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use bevy::ui_widgets::observe;

use super::display_case::UpdateDisplayCase;
use crate::prelude::*;

const HOVER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);
pub const VALUE_WIDTH: f32 = 60.0;
pub const EQUIP_SLOT_WIDTH: f32 = 150.0;

#[derive(Component)]
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

/// Spawns a given "slot" in a display case representing a single item in the inventory.
pub fn display_slot(icons: &GameIcons, context: DisplaySlotContext) -> impl Bundle {
    let equip_slot_string = context
        .equipment_slot
        .map_or("-".to_string(), |slot| slot.to_string());

    let equipped_icon = icons.equip.clone();
    let equipped = context.equipped;

    (
        DisplaySlotOf(context.item),
        Node {
            width: px(900.0),
            height: px(32.0),
            padding: px(5.0).all(),
            column_gap: px(5.0),
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
                    image: match context.kind {
                        ItemKind::Melee => icons.melee.clone(),
                        ItemKind::Staff => icons.staff.clone(),
                        ItemKind::Potion => icons.potion.clone(),
                        ItemKind::Tome => icons.spell_book.clone(),
                    },
                    ..default()
                },
                Node {
                    width: px(30.0),
                    height: px(30.0),
                    ..default()
                },
                Pickable::IGNORE,
            )),
            Spawn((text(context.name, 18.0), Pickable::IGNORE)),
            SpawnWith(move |parent: &mut ChildSpawner| {
                if equipped {
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
                text(context.value.to_string(), 18.0),
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
            height: px(16.0),
            width: px(16.0),
            ..default()
        },
        Pickable::IGNORE,
    ));
}

pub fn on_slot_clicked(
    pointer_click: On<Pointer<Click>>,
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
    pointer_over: On<Pointer<Over>>,
    mut item_slot: Query<&mut BackgroundColor, With<DisplaySlotOf>>,
) {
    if let Ok(mut background_color) = item_slot.get_mut(pointer_over.entity) {
        background_color.0 = HOVER_COLOR;
    }
}

pub fn on_slot_done_hovering(
    pointer_out: On<Pointer<Out>>,
    mut item_slot: Query<&mut BackgroundColor, With<DisplaySlotOf>>,
) {
    if let Ok(mut background_color) = item_slot.get_mut(pointer_out.entity) {
        background_color.0 = Color::NONE;
    }
}

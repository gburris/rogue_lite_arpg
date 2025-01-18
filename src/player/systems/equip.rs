use bevy::prelude::*;

//handle_equip_event
//Query player equipment
//Equip the entity from the equip_event to the player slot into it maps to
/*


*/
pub fn handle_potenequip_event(
    _: Trigger<EquipEvent>,
    mut commands: Commands,
    mut damaged_query: Query<(&mut Health, Option<&InvulnerableFromDamage>)>,
    source_query: Query<&EffectsList>,
) {


}

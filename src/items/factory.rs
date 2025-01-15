use crate::items::{Item, StatType};

pub fn get_default_staff() -> Item {
    let mut staff = Item::new("Staff of Casting");
    staff.add_stat(StatType::SpellPower, 10);
    staff.add_stat(StatType::CastSpeed, 10);

    return staff;
}

pub fn get_default_sword() -> Item {
    let mut staff = Item::new("Sword");
    staff.add_stat(StatType::AttackDamage, 10);

    return staff;
}

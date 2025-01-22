use bevy::{
    prelude::{Query, Res},
    time::Time,
};

use crate::combat::attributes::Mana;

pub fn regenerate_mana(mut query: Query<&mut Mana>, time: Res<Time>) {
    let delta_time = time.delta_secs();
    for mut mana in query.iter_mut() {
        mana.regenerate(delta_time);
    }
}

use bevy::prelude::*;

use crate::components::WarpZone;

pub fn debug_warpzone_transform(warpzone_query: Query<(Entity, &Transform), With<WarpZone>>) {
    for (entity, transform) in warpzone_query.iter() {
        println!(
            "Warpzone {:?} - Position: {:?}, Scale: {:?}",
            entity, transform.translation, transform.scale
        );
    }
}

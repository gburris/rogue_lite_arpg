use bevy::prelude::*;

use crate::{
    combat::status_effects::{Status, StatusApplied, StatusOf, slow::Slowed},
    utility::Lifespan,
};

#[derive(Component, Clone, Default)]
#[require(Status)]
pub struct Frozen;

const BLUE_COLOR: bevy::prelude::Color = Color::srgb(0.0, 0.0, 1.0);

pub fn apply_frozen(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusOf, &Lifespan), (With<Frozen>, Without<StatusApplied>)>,
    mut sprite_query: Query<&mut Sprite>,
) {
    status_query
        .iter()
        .for_each(|(status, status_of, duration)| {
            commands.entity(status).insert(StatusApplied);

            let slowed = commands
                .spawn((
                    Slowed {
                        percent: 1.0, // completely stun
                    },
                    // make sure slow lasts while frozen
                    Lifespan::new(duration.0.remaining_secs()),
                ))
                .id();

            commands
                .entity(status_of.0)
                .add_one_related::<StatusOf>(slowed);

            if let Ok(mut affected_sprite) = sprite_query.get_mut(status_of.0) {
                affected_sprite.color = BLUE_COLOR;
            }
        });
}

pub fn on_frozen_removed(
    frozen_status: On<Remove, Frozen>,
    status_query: Query<&StatusOf, With<Frozen>>,
    mut sprite_query: Query<&mut Sprite>,
) {
    let Ok(status_of) = status_query.get(frozen_status.entity) else {
        return;
    };

    if let Ok(mut affected_sprite) = sprite_query.get_mut(status_of.0) {
        affected_sprite.color = Color::default();
    }
}

use bevy::prelude::*;

use crate::{
    combat::status_effects::{Status, StatusApplied, StatusOf, slow::Slowed},
    prelude::*,
};

#[derive(Component, Clone, Default)]
#[require(Status)]
pub struct Frozen;

pub(super) fn apply_frozen(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusOf, &Lifespan), (With<Frozen>, Without<StatusApplied>)>,
    sprites: Res<SpriteAssets>,
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
                .add_one_related::<StatusOf>(slowed)
                .with_child(grounded_ice_vfx(&sprites, duration.0.remaining_secs()));
        });
}

fn grounded_ice_vfx(sprites: &SpriteAssets, duration: f32) -> impl Bundle {
    (
        Sprite::from_image(sprites.grounded_ice.clone()),
        Transform {
            translation: Vec3::new(
                0.0,
                CHARACTER_FEET_POS_OFFSET + 16.0,
                ZLayer::SpriteForeground.z(),
            ),
            scale: Vec3::new(1.4, 1.4, 1.0),
            ..default()
        },
        Lifespan::new(duration),
    )
}

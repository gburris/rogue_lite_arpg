use std::time::Duration;

use avian2d::prelude::CollidingEntities;

use bevy::{prelude::*, state::commands};

use crate::npc::events::{AttemptDialogueInput, DialogueBegin};

// Only query colliding entities with the NPCInteractionRadius component
// When it finds that they are in range, kick off a start dialogue trigger
pub fn handle_dialogue_input(
    _: Trigger<AttemptDialogueInput>,
    mut commands: Commands,
    query: Query<(Entity, &CollidingEntities)>,
) {
    for (entity, colliding_entities) in &query {
        warn!("Colliding entities: {:?}", colliding_entities);
        warn!("Entity: {:?}", entity);
        commands.trigger(DialogueBegin {
            entity: entity,
            colliding_entities: colliding_entities.clone(), //This is the NPC
        });
    }
}

//TODO: Replace all of this with a proper dialogue system
//Temp stuff to test this feature
#[derive(Component)]
pub struct DialogueBubble {
    timer: Timer,
    initial_alpha: f32,
}

// Triggers once the players presses interact in an NPCs interaction radius
// Insert a child entity with a dialogue box
// above the NPC saying "hello!"
pub fn begin_dialogue(
    dialogue_begin_trigger: Trigger<DialogueBegin>,
    mut commands: Commands,
    query: Query<&Transform>,
    camera_query: Query<(&Camera, &GlobalTransform)>, // Add camera query
) {
    // Get the camera and its transform
    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        // Get the first colliding entity
        if let Some(npc_entity) = dialogue_begin_trigger.colliding_entities.iter().next() {
            // Get the transform component for that entity
            if let Ok(npc_transform) = query.get(*npc_entity) {
                // Calculate position above NPC's head in world space
                let y_offset = 50.0;
                let world_pos = npc_transform.translation + Vec3::new(0.0, y_offset, 0.1);

                // Convert world position to screen space
                if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, world_pos) {
                    // Spawn the dialogue bubble as a UI element
                    commands
                        .spawn((
                            BackgroundColor::from(Color::WHITE),
                            BorderColor::from(Color::BLACK),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(screen_pos.x),
                                top: Val::Px(screen_pos.y),
                                padding: UiRect::all(Val::Px(10.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("You wanted something?"),
                                TextFont::default(),
                                TextColor::from(Color::BLACK),
                            ));
                        })
                        .insert(DialogueBubble {
                            timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
                            initial_alpha: 0.9,
                        });
                }
            }
        }
    }
}

// Update the dialogue bubbles system to handle screen space positioning
pub fn update_dialogue_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DialogueBubble, &mut BackgroundColor)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    npc_query: Query<&Transform>, // Query to get NPC positions
) {
    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        for (entity, mut bubble, mut background) in query.iter_mut() {
            bubble.timer.tick(time.delta());

            // Calculate fade based on remaining time
            let progress = bubble.timer.fraction();
            let alpha = bubble.initial_alpha * (1.0 - progress);

            // Update background transparency
            background.0.set_alpha(alpha);

            // Remove the bubble when timer is finished
            if bubble.timer.finished() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

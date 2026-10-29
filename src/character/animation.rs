use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_animation_state, cycle_character_animation)
            .chain()
            .in_set(InGameSystems::Vfx),
    );
}

#[derive(Component, Default, PartialEq, Debug, Hash, Eq, Copy, Clone)]
#[require(FacingDirection, Kinematics)]
pub(super) enum CharacterAnimationState {
    #[default]
    Idle,
    Moving,
    Attacking,
    Dying,
}

impl CharacterAnimationState {
    /// Row group index in [`AnimationSheet::animations`].
    fn index(self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Moving => 1,
            Self::Attacking => 2,
            Self::Dying => 3,
        }
    }
}

/// Sheet layout: columns plus one animation per state and facing direction.
#[derive(Component, Clone, Debug)]
pub(super) struct AnimationSheet {
    columns: usize,
    animations: [[AnimationData; 4]; 4],
    preserve_phase: bool,
}

impl Default for AnimationSheet {
    fn default() -> Self {
        // Rows are ordered Down, Right, Up, Left, matching the sheet.
        Self {
            columns: 13,
            animations: [
                state_rows([14, 15, 12, 13], 3, 0.5),
                state_rows([10, 11, 8, 9], 9, 0.1),
                state_rows([18, 19, 16, 17], 9, 0.1),
                state_rows([20, 20, 20, 20], 5, 0.4),
            ],
            preserve_phase: false,
        }
    }
}

impl AnimationSheet {
    /// Flapping flier layout sharing one wingbeat loop.
    pub(super) fn bat() -> Self {
        const BAT_FRAMES_PER_ROW: usize = 4;
        const BAT_FLAP_DURATION: f32 = 0.12;
        const BAT_ROWS: [usize; 4] = [0, 1, 2, 3];

        Self {
            columns: BAT_FRAMES_PER_ROW,
            animations: [
                state_rows(BAT_ROWS, BAT_FRAMES_PER_ROW, BAT_FLAP_DURATION),
                state_rows(BAT_ROWS, BAT_FRAMES_PER_ROW, BAT_FLAP_DURATION),
                state_rows(BAT_ROWS, BAT_FRAMES_PER_ROW, BAT_FLAP_DURATION),
                state_rows([0, 0, 0, 0], 1, BAT_FLAP_DURATION),
            ],
            preserve_phase: true,
        }
    }

    fn get(&self, state: CharacterAnimationState, direction: FacingDirection) -> &AnimationData {
        &self.animations[state.index()][direction_index(direction)]
    }
}

fn state_rows(rows: [usize; 4], frame_count: usize, frame_duration: f32) -> [AnimationData; 4] {
    rows.map(|row| AnimationData {
        row,
        frame_count,
        frame_duration,
    })
}

fn direction_index(direction: FacingDirection) -> usize {
    match direction {
        FacingDirection::Down => 0,
        FacingDirection::Right => 1,
        FacingDirection::Up => 2,
        FacingDirection::Left => 3,
    }
}

fn update_animation_state(
    mut character_query: Query<
        (
            &Kinematics,
            &AttackState,
            Option<&Health>,
            &mut CharacterAnimationState,
            &mut FacingDirection,
        ),
        Or<(Changed<Kinematics>, Changed<AttackState>, Changed<Health>)>,
    >,
) {
    for (kinematics, attack_state, health, mut animation_state, mut facing_direction) in
        &mut character_query
    {
        if health.is_none_or(|h| h.hp <= 0.0) {
            animation_state.set_if_neq(CharacterAnimationState::Dying);
            continue;
        }

        facing_direction.set_if_neq(FacingDirection::from_vec2(
            &facing_direction,
            kinematics.direction,
        ));

        // Attacking animation takes priority over walking / idle
        if attack_state.is_attacking {
            animation_state.set_if_neq(CharacterAnimationState::Attacking);
            continue;
        }

        if kinematics.is_moving() {
            animation_state.set_if_neq(CharacterAnimationState::Moving);
        } else {
            animation_state.set_if_neq(CharacterAnimationState::Idle);
        }
    }
}

fn cycle_character_animation(
    mut query: Query<
        (
            &mut AnimationIndices,
            &mut AnimationTimer,
            &mut Sprite,
            &CharacterAnimationState,
            &FacingDirection,
            &AnimationSheet,
        ),
        Or<(Changed<CharacterAnimationState>, Changed<FacingDirection>)>,
    >,
) {
    for (mut indices, mut timer, mut sprite, state, direction, sheet) in &mut query {
        let animation = sheet.get(*state, *direction);
        let first = animation.row * sheet.columns;
        let last = first + animation.frame_count - 1;

        // Row-major atlas: carry the frame column across a row change so a
        // shared loop keeps its beat instead of restarting on every turn.
        let offset = if sheet.preserve_phase {
            sprite
                .texture_atlas
                .as_ref()
                .map_or(0, |atlas| atlas.index % sheet.columns)
                % animation.frame_count
        } else {
            0
        };

        // A phase-preserving sheet also keeps its clock, or a facing flip
        // every frame would freeze the loop. Only a duration change re-arms it.
        let frame_duration = std::time::Duration::from_secs_f32(animation.frame_duration);
        if !sheet.preserve_phase || timer.duration() != frame_duration {
            *timer = AnimationTimer(Timer::from_seconds(
                animation.frame_duration,
                TimerMode::Repeating,
            ));
        }

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = first + offset;
        }

        // `Cycle::next` yields the range start first, so leave the iterator one
        // step ahead of the displayed frame.
        let mut cycle = (first..=last).cycle();
        for _ in 0..=offset {
            cycle.next();
        }
        *indices = AnimationIndices::Cycle(cycle);
    }
}

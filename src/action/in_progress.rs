use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::RngExt;

use crate::prelude::*;

/// Seconds an action source waits after a use interrupted during its wind-up before it can be used
/// again.
const INTERRUPTED_LOCKOUT_SECONDS: f32 = 1.5;

/// How long an action source waits after a use takes effect before it can be used again.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionCooldown {
    seconds: f32,
    jitter: f32,
    ready_at: f32,
}

impl ActionCooldown {
    /// Waits `seconds` after each use takes effect.
    pub fn new(seconds: f32) -> Self {
        Self {
            seconds,
            ..default()
        }
    }

    /// Varies each wait by up to `jitter` seconds either way, so characters that start using the
    /// same thing together drift out of step.
    pub fn with_jitter(self, jitter: f32) -> Self {
        Self { jitter, ..self }
    }

    /// Seconds left at virtual time `now` before the next use can start.
    pub fn remaining_seconds(&self, now: f32) -> f32 {
        (self.ready_at - now).max(0.0)
    }

    pub(super) fn is_ready(&self, now: f32) -> bool {
        now >= self.ready_at
    }

    fn start(&mut self, now: f32) {
        let offset = if self.jitter > 0.0 {
            rand::rng().random_range(-self.jitter..=self.jitter)
        } else {
            0.0
        };
        self.ready_at = now + (self.seconds + offset).max(0.0);
    }

    fn start_interrupted_lockout(&mut self, now: f32) {
        self.ready_at = self.ready_at.max(now + INTERRUPTED_LOCKOUT_SECONDS);
    }
}

/// How a use ends once it takes effect.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActionUseStyle {
    /// Ends the moment it takes effect, like a heal or a tome's spell.
    #[default]
    Instant,
    /// Plays out after it takes effect and ends itself, like a sword swing or a staff cast.
    Timed,
    /// Lasts until it is stopped, like a shield block.
    Sustained,
}

impl ActionUseStyle {
    /// Whether a use in this style ends without being stopped.
    pub fn ends_on_its_own(self) -> bool {
        match self {
            Self::Instant | Self::Timed => true,
            Self::Sustained => false,
        }
    }
}

/// How long each use winds up before it takes effect, a window for a hit to interrupt it. Without
/// it, a use takes effect on the frame it starts.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionWindup {
    pub seconds: f32,
}

/// Counts down a use's [`ActionWindup`].
#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub(super) struct ActionWindupProgress(Timer);

/// Lights the holder in this color while a use winds up, a stand-in until uses have animations.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionGlow(pub Color);

/// The light a winding-up use of an [`ActionGlow`] source shines on its holder.
#[derive(Component, Clone, Copy, Debug)]
#[component(on_remove = despawn_action_glow_light)]
pub(super) struct ActionGlowLight(Entity);

/// Present on an [`ActionSource`] while a use of it is under way, from the start of its wind-up
/// until the use ends as its [`ActionUseStyle`] says. An interrupted holder's use ends; see
/// [`InterruptionState`]. Its removal, however the use ended, fires [`ActionFinished`].
#[derive(Component, Clone, Copy, Debug)]
#[component(on_remove = report_action_finished)]
pub struct ActionInProgress {
    holder: Entity,
    slot: ActionSlot,
}

impl ActionInProgress {
    pub(super) fn new(holder: Entity, slot: ActionSlot) -> Self {
        Self { holder, slot }
    }

    /// The character using it.
    pub fn holder(&self) -> Entity {
        self.holder
    }

    /// The slot the holder uses it from.
    pub fn slot(&self) -> ActionSlot {
        self.slot
    }
}

/// A use of the action source `entity` took effect. The source's own observer carries out the
/// effect.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionTookEffect {
    pub entity: Entity,
    pub holder: Entity,
    pub slot: ActionSlot,
}

/// Request to end the use of `entity` in progress. Action sources whose effect leaves state behind,
/// such as a swing's hitbox, observe it to clean up.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct StopAction {
    pub entity: Entity,
}

/// Fired at the holder when one of its sources stops being in use, however the use ended.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionFinished {
    #[event_target]
    pub holder: Entity,
    pub entity: Entity,
    pub slot: ActionSlot,
}

/// Spends a new use's mana cost, then starts its [`ActionWindup`] or has it take effect at once.
pub(super) fn on_action_started(
    added: On<Add<ActionInProgress>>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut sources: Query<(
        &ActionInProgress,
        &ActionUseStyle,
        &mut ActionCooldown,
        Option<&ActionWindup>,
        Option<&ManaCost>,
    )>,
    mut holders: Query<&mut Mana>,
) {
    let Ok((&in_progress, &style, mut cooldown, windup, mana_cost)) = sources.get_mut(added.entity)
    else {
        return;
    };
    if let Some(&mana_cost) = mana_cost
        && let Ok(mut mana) = holders.get_mut(in_progress.holder)
    {
        mana.spend(mana_cost);
    }
    match windup {
        Some(windup) => {
            let timer = Timer::from_seconds(windup.seconds, TimerMode::Once);
            commands
                .entity(added.entity)
                .insert(ActionWindupProgress(timer));
        }
        None => take_effect(
            &mut commands,
            added.entity,
            in_progress,
            style,
            &mut cooldown,
            time.elapsed_secs(),
        ),
    }
}

/// Lights the holder of an [`ActionGlow`] source while its use winds up.
pub(super) fn on_action_windup_started(
    added: On<Add<ActionWindupProgress>>,
    mut commands: Commands,
    glows: Query<(&ActionGlow, &ActionInProgress)>,
) {
    let Ok((&ActionGlow(color), in_progress)) = glows.get(added.entity) else {
        return;
    };
    let holder = in_progress.holder;
    let light = commands
        .spawn_scene(bsn! {
            @point_light(color, 3.0, 40.0, 3.0)
            ChildOf(holder)
        })
        .id();
    commands.entity(added.entity).insert(ActionGlowLight(light));
}

/// Ends the use, whatever stage it is in.
pub(super) fn on_stop_action(stop: On<StopAction>, mut commands: Commands) {
    end_use(&mut commands, stop.entity);
}

/// Ends each use whose holder is interrupted (see [`InterruptionState`]), locking out one
/// interrupted during its wind-up, and has each finished wind-up take effect.
pub(super) fn advance_actions(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut sources: Query<(
        Entity,
        &ActionInProgress,
        &ActionUseStyle,
        &mut ActionCooldown,
        Option<&mut ActionWindupProgress>,
    )>,
    holders: Query<InterruptionState>,
) {
    let now = time.elapsed_secs();
    for (entity, &in_progress, &style, mut cooldown, windup_progress) in &mut sources {
        let holder_interrupted = holders
            .get(in_progress.holder)
            .ok()
            .is_none_or(|interruption| interruption.is_interrupted());
        if holder_interrupted {
            if windup_progress.is_some() {
                cooldown.start_interrupted_lockout(now);
            }
            commands.trigger(StopAction { entity });
        } else if let Some(mut windup_progress) = windup_progress
            && windup_progress.tick(time.delta()).is_finished()
        {
            take_effect(
                &mut commands,
                entity,
                in_progress,
                style,
                &mut cooldown,
                now,
            );
        }
    }
}

fn take_effect(
    commands: &mut Commands,
    entity: Entity,
    in_progress: ActionInProgress,
    style: ActionUseStyle,
    cooldown: &mut ActionCooldown,
    now: f32,
) {
    cooldown.start(now);
    commands.trigger(ActionTookEffect {
        entity,
        holder: in_progress.holder,
        slot: in_progress.slot,
    });
    match style {
        ActionUseStyle::Instant => end_use(commands, entity),
        ActionUseStyle::Timed | ActionUseStyle::Sustained => {
            commands
                .entity(entity)
                .remove::<(ActionWindupProgress, ActionGlowLight)>();
        }
    }
}

fn end_use(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .try_remove::<(ActionInProgress, ActionWindupProgress, ActionGlowLight)>();
}

fn report_action_finished(mut world: DeferredWorld, context: HookContext) {
    let Some(&ActionInProgress { holder, slot }) =
        world.entity(context.entity).get::<ActionInProgress>()
    else {
        return;
    };
    world.commands().trigger(ActionFinished {
        holder,
        entity: context.entity,
        slot,
    });
}

fn despawn_action_glow_light(mut world: DeferredWorld, context: HookContext) {
    let Some(&ActionGlowLight(light)) = world.entity(context.entity).get::<ActionGlowLight>()
    else {
        return;
    };
    world.commands().entity(light).try_despawn();
}

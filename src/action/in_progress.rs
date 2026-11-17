use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::query::QueryData;
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
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub enum ActionUseStyle {
    /// Ends the moment it takes effect, like a heal or a tome's spell.
    #[default]
    Instant,
    /// Plays out for `seconds` after it takes effect, then ends, like a sword swing or a staff
    /// cast.
    Timed { seconds: f32 },
    /// Lasts until it is stopped, like a shield block.
    Sustained,
}

impl ActionUseStyle {
    /// How long a timed use plays out; `None` for the other styles.
    pub fn timed_seconds(self) -> Option<f32> {
        match self {
            Self::Timed { seconds } => Some(seconds),
            Self::Instant | Self::Sustained => None,
        }
    }

    /// Whether a use in this style ends without being stopped.
    pub fn ends_on_its_own(self) -> bool {
        match self {
            Self::Instant | Self::Timed { .. } => true,
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

impl ActionWindupProgress {
    fn new(windup: ActionWindup) -> Self {
        Self(Timer::from_seconds(windup.seconds, TimerMode::Once))
    }
}

/// Counts down a timed use after it takes effect; see [`ActionUseStyle::Timed`].
#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub(super) struct TimedActionProgress(Timer);

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
    key: ActionKey,
}

impl ActionInProgress {
    pub(super) fn new(holder: Entity, key: ActionKey) -> Self {
        Self { holder, key }
    }

    /// The character using it.
    pub fn holder(&self) -> Entity {
        self.holder
    }

    /// The key the holder named it by.
    pub fn key(&self) -> ActionKey {
        self.key
    }
}

/// A use of the action `source` took effect. The source's own observer carries out the effect.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionTookEffect {
    #[event_target]
    pub source: Entity,
    pub holder: Entity,
    pub key: ActionKey,
}

/// Request to end the use of the action `source` in progress. Action sources whose effect leaves
/// state behind, such as a swing's hitbox, observe it to clean up.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct StopAction {
    #[event_target]
    pub source: Entity,
}

/// Fired at the holder when one of its sources stops being in use, however the use ended.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionFinished {
    #[event_target]
    pub holder: Entity,
    pub source: Entity,
    pub key: ActionKey,
}

/// What the pipeline reads and writes of an action source while a use of it is in progress.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct ActionUseInProgress {
    entity: Entity,
    in_progress: &'static ActionInProgress,
    style: &'static ActionUseStyle,
    cooldown: &'static mut ActionCooldown,
}

impl ActionUseInProgressItem<'_, '_> {
    /// Starts the cooldown and fires [`ActionTookEffect`]. An instant use ends here; any other
    /// use's wind-up state goes, and a timed use starts counting down.
    fn take_effect(&mut self, commands: &mut Commands, now: f32) {
        self.cooldown.start(now);
        commands.trigger(ActionTookEffect {
            source: self.entity,
            holder: self.in_progress.holder,
            key: self.in_progress.key,
        });
        match *self.style {
            ActionUseStyle::Instant => end_use(commands, self.entity),
            ActionUseStyle::Timed { seconds } => {
                let progress = Timer::from_seconds(seconds, TimerMode::Once);
                commands
                    .entity(self.entity)
                    .remove::<(ActionWindupProgress, ActionGlowLight)>()
                    .insert(TimedActionProgress(progress));
            }
            ActionUseStyle::Sustained => {
                commands
                    .entity(self.entity)
                    .remove::<(ActionWindupProgress, ActionGlowLight)>();
            }
        }
    }
}

/// Spends a new use's mana cost, then starts its [`ActionWindup`] or has it take effect at once.
pub(super) fn on_action_started(
    added: On<Add<ActionInProgress>>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut sources: Query<(
        ActionUseInProgress,
        Option<&ActionWindup>,
        Option<&ManaCost>,
    )>,
    mut holders: Query<&mut Mana>,
) {
    let Ok((mut use_in_progress, windup, mana_cost)) = sources.get_mut(added.entity) else {
        return;
    };
    if let Some(&mana_cost) = mana_cost
        && let Ok(mut mana) = holders.get_mut(use_in_progress.in_progress.holder)
    {
        mana.spend(mana_cost);
    }
    match windup {
        Some(windup) => {
            commands
                .entity(added.entity)
                .insert(ActionWindupProgress::new(*windup));
        }
        None => use_in_progress.take_effect(&mut commands, time.elapsed_secs()),
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
    end_use(&mut commands, stop.source);
}

/// Ends each use whose holder is interrupted (see [`InterruptionState`]), locking out one
/// interrupted during its wind-up, has each finished wind-up take effect, and ends each timed use
/// that has played out.
pub(super) fn advance_actions(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut sources: Query<(
        ActionUseInProgress,
        Option<&mut ActionWindupProgress>,
        Option<&mut TimedActionProgress>,
    )>,
    holders: Query<InterruptionState>,
) {
    let now = time.elapsed_secs();
    for (mut use_in_progress, windup_progress, timed_progress) in &mut sources {
        if is_holder_gone_or_interrupted(&holders, use_in_progress.in_progress.holder) {
            if windup_progress.is_some() {
                use_in_progress.cooldown.start_interrupted_lockout(now);
            }
            commands.trigger(StopAction {
                source: use_in_progress.entity,
            });
        } else if let Some(mut windup_progress) = windup_progress {
            if windup_progress.tick(time.delta()).is_finished() {
                use_in_progress.take_effect(&mut commands, now);
            }
        } else if let Some(mut timed_progress) = timed_progress
            && timed_progress.tick(time.delta()).is_finished()
        {
            commands.trigger(StopAction {
                source: use_in_progress.entity,
            });
        }
    }
}

fn is_holder_gone_or_interrupted(holders: &Query<InterruptionState>, holder: Entity) -> bool {
    holders
        .get(holder)
        .ok()
        .is_none_or(|interruption| interruption.is_interrupted())
}

fn end_use(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).try_remove::<(
        ActionInProgress,
        ActionWindupProgress,
        TimedActionProgress,
        ActionGlowLight,
    )>();
}

fn report_action_finished(mut world: DeferredWorld, context: HookContext) {
    let Some(&ActionInProgress { holder, key }) =
        world.entity(context.entity).get::<ActionInProgress>()
    else {
        return;
    };
    world.commands().trigger(ActionFinished {
        holder,
        source: context.entity,
        key,
    });
}

fn despawn_action_glow_light(mut world: DeferredWorld, context: HookContext) {
    let Some(&ActionGlowLight(light)) = world.entity(context.entity).get::<ActionGlowLight>()
    else {
        return;
    };
    world.commands().entity(light).try_despawn();
}

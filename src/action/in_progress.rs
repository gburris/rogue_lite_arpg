use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::query::QueryData;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

use super::action_status::ActionUseStyle;
use crate::prelude::*;

/// How long each use winds up before it takes effect, a window for a hit to interrupt it. Without
/// it, a use takes effect on the frame it starts.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionWindup {
    pub seconds: f32,
}

/// Lights the holder in this color while a use winds up.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionGlow(pub Color);

/// Seconds an action source waits after a use interrupted during its wind-up before it can be used
/// again.
const INTERRUPTED_LOCKOUT_SECONDS: f32 = 1.5;

/// Counts down a use's [`ActionWindup`].
#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub(super) struct ActionWindupProgress(Timer);

impl ActionWindupProgress {
    fn new(windup: ActionWindup) -> Self {
        Self(Timer::from_seconds(windup.seconds, TimerMode::Once))
    }
}

/// Counts down a timed use after it takes effect; see [`ActionUseStyle::Timed`]. Effects that play
/// out over the use, such as a swing, read how far along it is.
#[derive(Component, Clone, Debug, Deref)]
pub struct TimedActionProgress(Timer);

/// The light a winding-up use of an [`ActionGlow`] source shines on its holder.
#[derive(Component, Clone, Copy, Debug)]
#[component(on_remove = despawn_action_glow_light)]
struct ActionGlowLight(Entity);

/// Present on an [`ActionSource`] while a use of it is under way, from the start of its wind-up
/// until the use ends as its [`ActionUseStyle`] says. An interrupted holder's use ends; see
/// [`InterruptionState`]. Its removal, however the use ended, despawns the use's parts (see
/// [`PartOfActionUse`]) and fires [`ActionFinished`].
#[derive(Component, Clone, Copy, Debug)]
#[component(on_remove = end_action_use)]
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

/// Marks an entity, such as a hitbox, as part of the use of the action `source` in progress. It is
/// despawned when that use ends; see [`ActionInProgress`].
#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = ActionUseParts)]
pub struct PartOfActionUse {
    pub source: Entity,
}

/// The parts of the use of this action source in progress; see [`PartOfActionUse`].
#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = PartOfActionUse)]
pub struct ActionUseParts(Vec<Entity>);

/// A use of the action `source` took effect. The source's own observer carries out the effect.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionTookEffect {
    #[event_target]
    pub source: Entity,
    pub holder: Entity,
    pub key: ActionKey,
}

/// Request to end the use of the action `source` in progress, whatever stage it is in.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct StopAction {
    #[event_target]
    pub source: Entity,
}

/// Fired at the action `source` when its use ends, however it ended. Action sources whose effect
/// leaves state behind, such as a swing's hitbox, observe it to clean up.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionFinished {
    #[event_target]
    pub source: Entity,
    pub holder: Entity,
    pub key: ActionKey,
}

/// What the pipeline reads and writes of an action source while a use of it is in progress.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct ActionSourceInUse {
    entity: Entity,
    in_progress: &'static ActionInProgress,
    style: &'static ActionUseStyle,
    cooldown: &'static mut Cooldown,
}

impl ActionSourceInUseItem<'_, '_> {
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

/// Spends a new use's mana cost up front, so a use interrupted during its wind-up still costs it,
/// then starts its [`ActionWindup`] or has it take effect at once.
pub(super) fn on_action_started(
    added: On<Add<ActionInProgress>>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut sources: Query<(ActionSourceInUse, Option<&ActionWindup>, Option<&ManaCost>)>,
    mut holders: Query<&mut Mana>,
) {
    let Ok((mut source_in_use, windup, mana_cost)) = sources.get_mut(added.entity) else {
        return;
    };
    if let Some(&mana_cost) = mana_cost
        && let Ok(mut mana) = holders.get_mut(source_in_use.in_progress.holder)
    {
        mana.spend(mana_cost);
    }
    match windup {
        Some(windup) => {
            commands
                .entity(added.entity)
                .insert(ActionWindupProgress::new(*windup));
        }
        None => source_in_use.take_effect(&mut commands, time.elapsed_secs()),
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
        ActionSourceInUse,
        Option<&mut ActionWindupProgress>,
        Option<&mut TimedActionProgress>,
    )>,
    holders: Query<InterruptionState>,
) {
    let now = time.elapsed_secs();
    for (mut source_in_use, windup_progress, timed_progress) in &mut sources {
        if is_holder_gone_or_interrupted(&holders, source_in_use.in_progress.holder) {
            if windup_progress.is_some() {
                source_in_use
                    .cooldown
                    .lock_out(now, INTERRUPTED_LOCKOUT_SECONDS);
            }
            commands.trigger(StopAction {
                source: source_in_use.entity,
            });
        } else if let Some(mut windup_progress) = windup_progress {
            if windup_progress.tick(time.delta()).is_finished() {
                source_in_use.take_effect(&mut commands, now);
            }
        } else if let Some(mut timed_progress) = timed_progress
            && timed_progress.0.tick(time.delta()).is_finished()
        {
            commands.trigger(StopAction {
                source: source_in_use.entity,
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

fn end_action_use(mut world: DeferredWorld, context: HookContext) {
    let source = world.entity(context.entity);
    let Some(&ActionInProgress { holder, key }) = source.get::<ActionInProgress>() else {
        return;
    };

    let parts: Vec<Entity> = source
        .get::<ActionUseParts>()
        .map(|parts| parts.iter().collect())
        .unwrap_or_default();
    for part in parts {
        world.commands().entity(part).try_despawn();
    }

    world.commands().trigger(ActionFinished {
        source: context.entity,
        holder,
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

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

/// Makes this light the glow of a winding-up use; see [`ActionGlow`].
#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = ActionGlowLight)]
struct ActionGlowLightOf {
    action_use: Entity,
}

/// The light this use's [`ActionGlow`] shines on its holder while it winds up; see
/// [`ActionGlowLightOf`].
#[derive(Component, Debug)]
#[relationship_target(relationship = ActionGlowLightOf, linked_spawn)]
struct ActionGlowLight(Entity);

/// A use of an [`ActionSource`] under way, on its own entity: spawned when the use starts and
/// despawned when it ends as its [`ActionUseStyle`] says. An interrupted holder's use ends; see
/// [`InterruptionState`]. Its removal, however the use ended, fires [`ActionFinished`].
#[derive(Component, Clone, Copy, Debug)]
#[component(on_remove = report_action_finished)]
pub(super) struct ActionUse {
    key: ActionKey,
}

impl ActionUse {
    pub(super) fn new(key: ActionKey) -> Self {
        Self { key }
    }
}

/// Makes this use entity a use by the character `holder`.
#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = ActionUses)]
pub struct ActionUseOf {
    pub holder: Entity,
}

/// The uses this character has under way; see [`ActionUseOf`].
#[derive(Component, Debug)]
#[relationship_target(relationship = ActionUseOf, linked_spawn)]
pub struct ActionUses(Vec<Entity>);

/// Makes this use entity the use of the action `source` under way.
#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = ActionSourceUse)]
pub struct ActionSourceUseOf {
    pub source: Entity,
}

/// The use of this action source under way, present only while one is; see
/// [`ActionSourceUseOf`]. Its arrival on the source starts the use.
#[derive(Component, Debug)]
#[relationship_target(relationship = ActionSourceUseOf, linked_spawn)]
pub struct ActionSourceUse(Entity);

impl ActionSourceUse {
    /// The use entity.
    pub fn get(&self) -> Entity {
        self.0
    }
}

/// Marks an entity, such as a hitbox, as part of the use `action_use`. It is despawned when that
/// use ends.
#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = ActionUseParts)]
pub struct ActionUsePartOf {
    pub action_use: Entity,
}

/// The parts of this use; see [`ActionUsePartOf`].
#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = ActionUsePartOf, linked_spawn)]
pub struct ActionUseParts(Vec<Entity>);

/// The use `action_use` of the action `source` took effect. The source's own observer carries out
/// the effect, linking any entities it spawns for the use to `action_use`; see [`ActionUsePartOf`].
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionTookEffect {
    #[event_target]
    pub source: Entity,
    pub action_use: Entity,
    pub holder: Entity,
    pub key: ActionKey,
}

/// Request to end the use of the action `source` under way, whatever stage it is in.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct StopAction {
    #[event_target]
    pub source: Entity,
}

/// Fired at the action `source` when its use ends, however it ended. Action sources whose effect
/// leaves state behind on the source, such as a swing's pose, observe it to clean up.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionFinished {
    #[event_target]
    pub source: Entity,
    pub holder: Entity,
    pub key: ActionKey,
}

/// What the pipeline reads of a use entity.
#[derive(QueryData)]
pub(super) struct ActionUseData {
    entity: Entity,
    action_use: &'static ActionUse,
    use_of: &'static ActionUseOf,
    source_use_of: &'static ActionSourceUseOf,
    glow_light: Option<&'static ActionGlowLight>,
}

impl ActionUseDataItem<'_, '_> {
    /// Starts the source's cooldown and fires [`ActionTookEffect`]. An instant use ends here; any
    /// other use's wind-up state goes, and a timed use starts counting down.
    fn take_effect(
        &self,
        commands: &mut Commands,
        style: ActionUseStyle,
        cooldown: &mut Cooldown,
        now: f32,
    ) {
        cooldown.start(now);
        commands.trigger(ActionTookEffect {
            source: self.source_use_of.source,
            action_use: self.entity,
            holder: self.use_of.holder,
            key: self.action_use.key,
        });
        if style == ActionUseStyle::Instant {
            commands.entity(self.entity).try_despawn();
            return;
        }
        if let Some(glow_light) = self.glow_light {
            commands.entity(glow_light.0).try_despawn();
        }
        let mut action_use = commands.entity(self.entity);
        action_use.remove::<ActionWindupProgress>();
        if let ActionUseStyle::Timed { seconds } = style {
            let progress = Timer::from_seconds(seconds, TimerMode::Once);
            action_use.insert(TimedActionProgress(progress));
        }
    }
}

/// Spends a new use's mana cost up front, so a use interrupted during its wind-up still costs it,
/// then starts its [`ActionWindup`] or has it take effect at once.
pub(super) fn on_action_started(
    added: On<Add<ActionSourceUse>>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    action_uses: Query<ActionUseData>,
    mut sources: Query<(
        &ActionSourceUse,
        &ActionUseStyle,
        &mut Cooldown,
        Option<&ActionWindup>,
        Option<&ManaCost>,
    )>,
    mut holders: Query<&mut Mana>,
) {
    let Ok((source_use, &style, mut cooldown, windup, mana_cost)) = sources.get_mut(added.entity)
    else {
        return;
    };
    let Ok(action_use) = action_uses.get(source_use.get()) else {
        return;
    };
    if let Some(&mana_cost) = mana_cost
        && let Ok(mut mana) = holders.get_mut(action_use.use_of.holder)
    {
        mana.spend(mana_cost);
    }
    match windup {
        Some(windup) => {
            commands
                .entity(action_use.entity)
                .insert(ActionWindupProgress::new(*windup));
        }
        None => action_use.take_effect(&mut commands, style, &mut cooldown, time.elapsed_secs()),
    }
}

/// Lights the holder of an [`ActionGlow`] source while its use winds up.
pub(super) fn on_action_windup_started(
    added: On<Add<ActionWindupProgress>>,
    mut commands: Commands,
    action_uses: Query<(&ActionUseOf, &ActionSourceUseOf)>,
    glows: Query<&ActionGlow>,
) {
    let Ok((&ActionUseOf { holder }, source_use_of)) = action_uses.get(added.entity) else {
        return;
    };
    let Ok(&ActionGlow(color)) = glows.get(source_use_of.source) else {
        return;
    };
    let action_use = added.entity;
    commands.spawn_scene(bsn! {
        @point_light(color, 3.0, 40.0, 3.0)
        ChildOf(holder)
        ~{ActionGlowLightOf { action_use }}
    });
}

pub(super) fn on_stop_action(
    stop: On<StopAction>,
    mut commands: Commands,
    sources: Query<&ActionSourceUse>,
) {
    if let Ok(source_use) = sources.get(stop.source) {
        commands.entity(source_use.get()).try_despawn();
    }
}

/// Ends each use whose holder is interrupted (see [`InterruptionState`]), locking out one
/// interrupted during its wind-up, has each finished wind-up take effect, and ends each timed use
/// that has played out.
pub(super) fn advance_actions(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut action_uses: Query<(
        ActionUseData,
        Option<&mut ActionWindupProgress>,
        Option<&mut TimedActionProgress>,
    )>,
    mut sources: Query<(&ActionUseStyle, &mut Cooldown)>,
    holders: Query<InterruptionState>,
) {
    let now = time.elapsed_secs();
    for (action_use, windup_progress, timed_progress) in &mut action_uses {
        let Ok((&style, mut cooldown)) = sources.get_mut(action_use.source_use_of.source) else {
            continue;
        };
        if is_holder_gone_or_interrupted(&holders, action_use.use_of.holder) {
            if windup_progress.is_some() {
                cooldown.lock_out(now, INTERRUPTED_LOCKOUT_SECONDS);
            }
            commands.entity(action_use.entity).try_despawn();
        } else if let Some(mut windup_progress) = windup_progress {
            if windup_progress.tick(time.delta()).is_finished() {
                action_use.take_effect(&mut commands, style, &mut cooldown, now);
            }
        } else if let Some(mut timed_progress) = timed_progress
            && timed_progress.0.tick(time.delta()).is_finished()
        {
            commands.entity(action_use.entity).try_despawn();
        }
    }
}

fn is_holder_gone_or_interrupted(holders: &Query<InterruptionState>, holder: Entity) -> bool {
    holders
        .get(holder)
        .ok()
        .is_none_or(|interruption| interruption.is_interrupted())
}

fn report_action_finished(mut world: DeferredWorld, context: HookContext) {
    let Ok((&ActionUse { key }, &ActionUseOf { holder }, &ActionSourceUseOf { source })) = world
        .entity(context.entity)
        .get_components::<(&ActionUse, &ActionUseOf, &ActionSourceUseOf)>()
    else {
        return;
    };
    world.commands().trigger(ActionFinished {
        source,
        holder,
        key,
    });
}

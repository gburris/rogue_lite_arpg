use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::RngExt;

use crate::prelude::*;

/// Seconds a usable waits after a use interrupted during its wind-up before it can be used again.
const INTERRUPTED_LOCKOUT_SECONDS: f32 = 1.5;

/// How long a usable waits after a use takes effect before it can be used again.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct UseCooldown {
    seconds: f32,
    jitter: f32,
    ready_at: f32,
}

impl UseCooldown {
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
pub enum UseStyle {
    /// Ends the moment it takes effect, like a heal or a tome's spell.
    #[default]
    Instant,
    /// Plays out after it takes effect and ends itself, like a sword swing or a staff cast.
    Timed,
    /// Lasts until it is stopped, like a shield block.
    Sustained,
}

impl UseStyle {
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
pub struct UseWindup {
    pub seconds: f32,
}

/// Counts down a use's [`UseWindup`].
#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub(super) struct UseWindupProgress(Timer);

/// Lights the holder in this color while a use winds up, a stand-in until uses have animations.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct UseGlow(pub Color);

/// The light a winding-up use of a [`UseGlow`] usable shines on its holder.
#[derive(Component, Clone, Copy, Debug)]
#[component(on_remove = despawn_use_glow_light)]
pub(super) struct UseGlowLight(Entity);

/// Present on a [`Usable`] while a use of it is under way, from the start of its wind-up until the
/// use ends as its [`UseStyle`] says. An interrupted holder's use ends; see
/// [`InterruptionState`]. Its removal, however the use ended, fires [`UseFinished`].
#[derive(Component, Clone, Copy, Debug)]
#[component(on_remove = report_use_finished)]
pub struct InUse {
    holder: Entity,
    slot: UseSlot,
}

impl InUse {
    pub(super) fn new(holder: Entity, slot: UseSlot) -> Self {
        Self { holder, slot }
    }

    /// The character using it.
    pub fn holder(&self) -> Entity {
        self.holder
    }

    /// The slot the holder uses it from.
    pub fn slot(&self) -> UseSlot {
        self.slot
    }
}

/// A use of the usable `entity` took effect. The usable's own observer carries out the effect.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct UseTookEffect {
    pub entity: Entity,
    pub holder: Entity,
    pub slot: UseSlot,
}

/// Request to end the use of `entity` in progress. Usables whose effect leaves state behind, such
/// as a swing's hitbox, observe it to clean up.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct StopUse {
    pub entity: Entity,
}

/// Fired at the holder when one of its usables stops being in use, however the use ended.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct UseFinished {
    #[event_target]
    pub holder: Entity,
    pub entity: Entity,
    pub slot: UseSlot,
}

/// Spends a new use's mana cost, then starts its [`UseWindup`] or has it take effect at once.
pub(super) fn on_use_started(
    added: On<Add<InUse>>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut uses: Query<(
        &InUse,
        &UseStyle,
        &mut UseCooldown,
        Option<&UseWindup>,
        Option<&ManaCost>,
    )>,
    mut holders: Query<&mut Mana>,
) {
    let Ok((&in_use, &style, mut cooldown, windup, mana_cost)) = uses.get_mut(added.entity) else {
        return;
    };
    if let Some(&mana_cost) = mana_cost
        && let Ok(mut mana) = holders.get_mut(in_use.holder)
    {
        mana.spend(mana_cost);
    }
    match windup {
        Some(windup) => {
            let timer = Timer::from_seconds(windup.seconds, TimerMode::Once);
            commands
                .entity(added.entity)
                .insert(UseWindupProgress(timer));
        }
        None => take_effect(
            &mut commands,
            added.entity,
            in_use,
            style,
            &mut cooldown,
            time.elapsed_secs(),
        ),
    }
}

/// Lights the holder of a [`UseGlow`] usable while its use winds up.
pub(super) fn on_use_windup_started(
    added: On<Add<UseWindupProgress>>,
    mut commands: Commands,
    glows: Query<(&UseGlow, &InUse)>,
) {
    let Ok((&UseGlow(color), in_use)) = glows.get(added.entity) else {
        return;
    };
    let holder = in_use.holder;
    let light = commands
        .spawn_scene(bsn! {
            @point_light(color, 3.0, 40.0, 3.0)
            ChildOf(holder)
        })
        .id();
    commands.entity(added.entity).insert(UseGlowLight(light));
}

/// Ends the use, whatever stage it is in.
pub(super) fn on_stop_use(stop: On<StopUse>, mut commands: Commands) {
    end_use(&mut commands, stop.entity);
}

/// Ends each use whose holder is interrupted (see [`InterruptionState`]), locking out one
/// interrupted during its wind-up, and has each finished wind-up take effect.
pub(super) fn advance_uses(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut uses: Query<(
        Entity,
        &InUse,
        &UseStyle,
        &mut UseCooldown,
        Option<&mut UseWindupProgress>,
    )>,
    holders: Query<InterruptionState>,
) {
    let now = time.elapsed_secs();
    for (entity, &in_use, &style, mut cooldown, windup_progress) in &mut uses {
        let holder_interrupted = holders
            .get(in_use.holder)
            .ok()
            .is_none_or(|interruption| interruption.is_interrupted());
        if holder_interrupted {
            if windup_progress.is_some() {
                cooldown.start_interrupted_lockout(now);
            }
            commands.trigger(StopUse { entity });
        } else if let Some(mut windup_progress) = windup_progress
            && windup_progress.tick(time.delta()).is_finished()
        {
            take_effect(&mut commands, entity, in_use, style, &mut cooldown, now);
        }
    }
}

fn take_effect(
    commands: &mut Commands,
    entity: Entity,
    in_use: InUse,
    style: UseStyle,
    cooldown: &mut UseCooldown,
    now: f32,
) {
    cooldown.start(now);
    commands.trigger(UseTookEffect {
        entity,
        holder: in_use.holder,
        slot: in_use.slot,
    });
    match style {
        UseStyle::Instant => end_use(commands, entity),
        UseStyle::Timed | UseStyle::Sustained => {
            commands
                .entity(entity)
                .remove::<(UseWindupProgress, UseGlowLight)>();
        }
    }
}

fn end_use(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .try_remove::<(InUse, UseWindupProgress, UseGlowLight)>();
}

fn report_use_finished(mut world: DeferredWorld, context: HookContext) {
    let Some(&InUse { holder, slot }) = world.entity(context.entity).get::<InUse>() else {
        return;
    };
    world.commands().trigger(UseFinished {
        holder,
        entity: context.entity,
        slot,
    });
}

fn despawn_use_glow_light(mut world: DeferredWorld, context: HookContext) {
    let Some(&UseGlowLight(light)) = world.entity(context.entity).get::<UseGlowLight>() else {
        return;
    };
    world.commands().entity(light).try_despawn();
}

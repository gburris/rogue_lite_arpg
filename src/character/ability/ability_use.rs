use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rand::RngExt;

use super::Ability;
use super::bite::Bite;
use super::breath::Breath;
use super::heal::find_carried_potion;
use super::rally::Rally;
use crate::prelude::*;

/// Seconds an ability waits after an interrupted use before it can be used again.
const INTERRUPTED_LOCKOUT_SECONDS: f32 = 1.5;

/// Links an ability entity to the character that uses it.
#[derive(Component, Clone)]
#[relationship(relationship_target = Abilities)]
pub struct AbilityOf {
    pub holder: Entity,
}

/// A character's ability entities, each carrying one capability component and whatever it
/// applies or fires.
#[derive(Component, Clone)]
#[relationship_target(relationship = AbilityOf, linked_spawn)]
pub struct Abilities(Vec<Entity>);

/// How long an ability waits after it takes effect before it can be used again.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct AbilityCooldown {
    seconds: f32,
    jitter: f32,
    ready_at: f32,
}

impl AbilityCooldown {
    /// Waits `seconds` after each use takes effect.
    pub fn new(seconds: f32) -> Self {
        Self {
            seconds,
            ..default()
        }
    }

    /// Varies each wait by up to `jitter` seconds either way, so characters that start using the
    /// ability together drift out of step.
    pub fn with_jitter(self, jitter: f32) -> Self {
        Self { jitter, ..self }
    }

    fn is_ready(&self, now: f32) -> bool {
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

/// One step of an ability's use; see [`AbilitySteps`].
#[derive(Clone, Copy, Debug)]
pub enum AbilityStep {
    /// Waits before the next step.
    Hold { seconds: f32 },
    /// The ability takes effect and starts its [`AbilityCooldown`].
    Release,
}

/// The steps each use of an ability runs through, in order. By default it takes effect at once.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct AbilitySteps(pub &'static [AbilityStep]);

impl Default for AbilitySteps {
    fn default() -> Self {
        Self(&[AbilityStep::Release])
    }
}

/// Lights the holder in this color while the ability is in use, a stand-in until abilities have
/// animations.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct UseGlow(pub Color);

/// Present on an ability entity while a use runs its [`AbilitySteps`]. A holder that is knocked
/// back, stunned or defeated can't start a use and cancels one under way, after which the ability
/// waits [`INTERRUPTED_LOCKOUT_SECONDS`]. Its removal, however the use ended, fires
/// [`AbilityUseFinished`].
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct AbilityInUse {
    step: usize,
    held_seconds: f32,
    glow_light: Option<Entity>,
}

/// A use of the ability entity `entity` reached its [`AbilityStep::Release`]. The observer of
/// its capability component applies the effect.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(super) struct AbilityReleased {
    pub entity: Entity,
}

/// Fired at the holder when one of its abilities stops being in use, however the use ended.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct AbilityUseFinished {
    #[event_target]
    pub holder: Entity,
    pub ability: Ability,
}

/// Why a character can't use an ability now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbilityUseFailure {
    /// The character has no ability entity for it.
    NotOwned,
    /// A heal with no potion to drink.
    NoPotion,
    /// The character is knocked back, stunned or defeated.
    HolderDisabled,
    InUse,
    OnCooldown,
}

impl AbilityUseFailure {
    /// Whether the ability becomes usable again by waiting, rather than needing something the
    /// character lacks.
    pub fn is_temporary(self) -> bool {
        match self {
            Self::HolderDisabled | Self::InUse | Self::OnCooldown => true,
            Self::NotOwned | Self::NoPotion => false,
        }
    }
}

/// Whether a holder is able to use its abilities; see [`AbilityInUse`].
#[derive(QueryData)]
pub(super) struct HolderState {
    health: Option<&'static Health>,
    stat_multipliers: Option<&'static StatMultipliers>,
    knocked_back: Has<KnockedBack>,
}

impl HolderStateItem<'_, '_> {
    fn is_disabled(&self) -> bool {
        self.knocked_back
            || self.health.is_some_and(Health::is_dead)
            || self
                .stat_multipliers
                .is_some_and(|multipliers| multipliers.is_stunned())
    }
}

/// What [`AbilityRead`] reads of one ability entity.
#[derive(QueryData)]
struct AbilityState {
    entity: Entity,
    ability: &'static Ability,
    cooldown: &'static AbilityCooldown,
    in_use: Has<AbilityInUse>,
    bite: Option<&'static Bite>,
    breath: Option<&'static Breath>,
    rally: Option<&'static Rally>,
}

impl AbilityStateItem<'_, '_> {
    fn reach(&self) -> Option<f32> {
        match self.ability {
            Ability::Heal => None,
            Ability::Breathe => self.breath.copied().map(Breath::reach),
            Ability::Bite => self.bite.map(Bite::reach),
            Ability::Rally => self.rally.copied().map(Rally::reach),
        }
    }
}

/// Reads which abilities a character has and whether it can use each now.
#[derive(SystemParam)]
pub struct AbilityRead<'w, 's> {
    time: Res<'w, Time<Virtual>>,
    holders: Query<'w, 's, (&'static Abilities, Option<&'static Items>, HolderState)>,
    abilities: Query<'w, 's, AbilityState>,
    consumables: Query<'w, 's, (), With<Consumable>>,
}

impl AbilityRead<'_, '_> {
    /// Whether `holder` has `ability`.
    pub fn has(&self, holder: Entity, ability: Ability) -> bool {
        self.find(holder, ability).is_some()
    }

    /// How far `holder`'s `ability` reaches; `None` for an ability without a reach or one it
    /// doesn't have.
    pub fn reach(&self, holder: Entity, ability: Ability) -> Option<f32> {
        self.find(holder, ability)?.reach()
    }

    /// The ability entity `holder` would use for `ability` now, or why it can't.
    pub fn can_use(&self, holder: Entity, ability: Ability) -> Result<Entity, AbilityUseFailure> {
        let state = self
            .find(holder, ability)
            .ok_or(AbilityUseFailure::NotOwned)?;
        let (_, items, holder_state) = self
            .holders
            .get(holder)
            .map_err(|_| AbilityUseFailure::NotOwned)?;
        if ability == Ability::Heal && find_carried_potion(items, &self.consumables).is_none() {
            return Err(AbilityUseFailure::NoPotion);
        }
        if holder_state.is_disabled() {
            return Err(AbilityUseFailure::HolderDisabled);
        }
        if state.in_use {
            return Err(AbilityUseFailure::InUse);
        }
        if !state.cooldown.is_ready(self.time.elapsed_secs()) {
            return Err(AbilityUseFailure::OnCooldown);
        }
        Ok(state.entity)
    }

    fn find(&self, holder: Entity, ability: Ability) -> Option<AbilityStateItem<'_, '_>> {
        let (abilities, ..) = self.holders.get(holder).ok()?;
        abilities
            .iter()
            .filter_map(|entity| self.abilities.get(entity).ok())
            .find(|state| *state.ability == ability)
    }
}

/// Starts ability uses.
#[derive(SystemParam)]
pub struct AbilityUsage<'w, 's> {
    commands: Commands<'w, 's>,
    read: AbilityRead<'w, 's>,
}

impl AbilityUsage<'_, '_> {
    /// Starts a use of `holder`'s `ability` if [`AbilityRead::can_use`] allows it.
    pub fn try_use(&mut self, holder: Entity, ability: Ability) -> Result<(), AbilityUseFailure> {
        let entity = self.read.can_use(holder, ability)?;
        self.commands
            .entity(entity)
            .insert_if_new(AbilityInUse::default());
        Ok(())
    }

    /// See [`AbilityRead::reach`].
    pub fn reach(&self, holder: Entity, ability: Ability) -> Option<f32> {
        self.read.reach(holder, ability)
    }
}

/// Runs a new use's first steps at once, so an ability without a hold takes effect on the frame
/// it is used, and lights the holder for a use that goes on.
pub(super) fn on_ability_use_started(
    added: On<Add<AbilityInUse>>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut uses: Query<(
        &mut AbilityInUse,
        &AbilitySteps,
        &mut AbilityCooldown,
        &AbilityOf,
        Option<&UseGlow>,
    )>,
) {
    let Ok((mut in_use, steps, mut cooldown, ability_of, glow)) = uses.get_mut(added.entity) else {
        return;
    };
    let finished = advance_steps(
        &mut commands,
        added.entity,
        &mut in_use,
        steps,
        &mut cooldown,
        time.elapsed_secs(),
        0.0,
    );
    if let Some(&UseGlow(color)) = glow.filter(|_| !finished) {
        let holder = ability_of.holder;
        let light = commands
            .spawn_scene(bsn! {
                @point_light(color, 3.0, 40.0, 3.0)
                ChildOf(holder)
            })
            .id();
        in_use.glow_light = Some(light);
    }
}

/// Advances every use under way, and cancels the use of a disabled holder; see [`AbilityInUse`].
pub(super) fn advance_ability_uses(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut uses: Query<(
        Entity,
        &mut AbilityInUse,
        &AbilitySteps,
        &mut AbilityCooldown,
        &AbilityOf,
    )>,
    holders: Query<HolderState>,
) {
    let now = time.elapsed_secs();
    for (entity, mut in_use, steps, mut cooldown, ability_of) in &mut uses {
        let holder_disabled = holders
            .get(ability_of.holder)
            .ok()
            .is_none_or(|holder_state| holder_state.is_disabled());
        if holder_disabled {
            cancel_ability_use(&mut commands, entity, &mut cooldown, now);
            continue;
        }
        advance_steps(
            &mut commands,
            entity,
            &mut in_use,
            steps,
            &mut cooldown,
            now,
            time.delta_secs(),
        );
    }
}

/// Puts out a finished use's glow and reports [`AbilityUseFinished`].
pub(super) fn on_ability_use_ended(
    removed: On<Remove<AbilityInUse>>,
    mut commands: Commands,
    uses: Query<(&AbilityInUse, &Ability, &AbilityOf)>,
) {
    let Ok((in_use, &ability, ability_of)) = uses.get(removed.entity) else {
        return;
    };
    if let Some(light) = in_use.glow_light {
        commands.entity(light).try_despawn();
    }
    commands.trigger(AbilityUseFinished {
        holder: ability_of.holder,
        ability,
    });
}

fn advance_steps(
    commands: &mut Commands,
    entity: Entity,
    in_use: &mut AbilityInUse,
    steps: &AbilitySteps,
    cooldown: &mut AbilityCooldown,
    now: f32,
    mut delta_seconds: f32,
) -> bool {
    loop {
        match steps.get(in_use.step) {
            None => {
                commands.entity(entity).remove::<AbilityInUse>();
                return true;
            }
            Some(AbilityStep::Release) => {
                cooldown.start(now);
                commands.trigger(AbilityReleased { entity });
            }
            Some(&AbilityStep::Hold { seconds }) => {
                in_use.held_seconds += delta_seconds;
                delta_seconds = 0.0;
                if in_use.held_seconds < seconds {
                    return false;
                }
                in_use.held_seconds = 0.0;
            }
        }
        in_use.step += 1;
    }
}

fn cancel_ability_use(
    commands: &mut Commands,
    entity: Entity,
    cooldown: &mut AbilityCooldown,
    now: f32,
) {
    cooldown.start_interrupted_lockout(now);
    commands.entity(entity).remove::<AbilityInUse>();
}

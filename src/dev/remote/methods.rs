use std::fmt::Debug;

use bevy::prelude::*;
use bevy::remote::builtin_methods::parse;
use bevy::remote::{BrpError, BrpResult, RemotePlugin, error_codes};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{explain, input, session, setup, snapshot};

/// One `game/…` method as `game/help` lists it.
#[derive(Clone, Copy, Debug)]
pub(super) struct MethodHelp {
    name: &'static str,
    params: &'static str,
    summary: &'static str,
}

/// Every `game/…` method's help, in the order they were registered.
#[derive(Resource, Debug, Deref)]
pub(super) struct GameMethodHelp(Vec<MethodHelp>);

/// Builds the remote plugin's `game/…` methods together with the help text `game/help` serves,
/// so a method can't be registered without describing itself.
pub(super) struct GameMethods {
    remote_plugin: RemotePlugin,
    help: Vec<MethodHelp>,
}

impl GameMethods {
    fn new() -> Self {
        Self {
            remote_plugin: RemotePlugin::default(),
            help: Vec::new(),
        }
    }

    /// Registers `handler` as the method `name`. `params` lists the method's params the way
    /// a caller writes them, with `?` after the optional ones.
    fn method<M>(
        mut self,
        name: &'static str,
        params: &'static str,
        summary: &'static str,
        handler: impl IntoSystem<In<Option<Value>>, BrpResult, M>,
    ) -> Self {
        self.remote_plugin = self.remote_plugin.with_method_main(name, handler);
        self.help.push(MethodHelp {
            name,
            params,
            summary,
        });
        self
    }

    pub(super) fn into_plugin_and_help(self) -> (RemotePlugin, GameMethodHelp) {
        (self.remote_plugin, GameMethodHelp(self.help))
    }
}

/// Every `game/…` method, with its help.
pub(super) fn game_methods() -> GameMethods {
    let methods = GameMethods::new();
    let methods = add_session_methods(methods);
    let methods = add_observation_methods(methods);
    let methods = add_input_methods(methods);
    add_setup_methods(methods)
}

fn add_session_methods(methods: GameMethods) -> GameMethods {
    methods
        .method(
            "game/help",
            "",
            "Lists every game/… method with its params and what it does.",
            list_methods,
        )
        .method(
            "game/start",
            "",
            "Leaves the start screen, as its button does.",
            session::start_game,
        )
        .method(
            "game/load_map",
            "{recipe, seed?}",
            "Builds a map recipe by name, replacing the current map.",
            session::load_map,
        )
        .method(
            "game/time",
            "{paused?, speed?}",
            "Pauses, resumes or rescales game time, then returns it. Physics and fixed steps \
             follow; menus and real-time tools don't.",
            session::control_time,
        )
        .method(
            "game/screenshot",
            "{path}",
            "Saves the primary window to an image file a frame or two later.",
            session::take_screenshot,
        )
        .method(
            "game/exit",
            "",
            "Closes the game after this frame.",
            session::exit_game,
        )
}

fn add_observation_methods(methods: GameMethods) -> GameMethods {
    methods
        .method(
            "game/snapshot",
            "{hits_after?}",
            "The frame number, app and menu states, game time, the current map (its entity \
             changes with every build), the player and their inventory, every enemy's brain, \
             movement and perception, villagers, projectiles, items on the ground, interactables, \
             portals and the latest hits. Each hit has a `sequence`; pass a snapshot's `last_hit` \
             back as `hits_after` to get only the hits landed since that snapshot.",
            snapshot::take_snapshot,
        )
        .method(
            "game/explain",
            "{entity?}",
            "How each of a brain's behaviors scored at its last decision, with every \
             consideration's input and score, or why the behavior sat out. Without `entity`, \
             returns a list with every brain. The first call on a brain starts tracing it and \
             returns no behaviors; it stays traced until it despawns.",
            explain::explain_brains,
        )
        .method(
            "game/ui",
            "",
            "Every visible UI text with its center and size in logical window pixels, for \
             game/click.",
            input::list_ui_text,
        )
}

fn add_input_methods(methods: GameMethods) -> GameMethods {
    methods
        .method(
            "game/player_input",
            "{move?, aim?, mainhand?, offhand?}",
            "Drives the player's own input actions, so the game reacts as it does to the keyboard \
             and mouse. `move` and `offhand` stay held until the next call, and a call without \
             them lets go. `mainhand` is one click per call, as long as a frame passes between \
             calls. `aim` moves the reticle to an offset from the player, at most 120 px away \
             as with the mouse.",
            input::set_player_input,
        )
        .method(
            "game/key",
            "{key, action}",
            "Presses, releases or taps a key as if typed into the window, so every binding works, \
             such as \"Space\" to interact, \"KeyI\" for the inventory, \"Escape\" to pause and \
             the dev keys. `key` is a `KeyCode` name and `action` is `press`, `release` or `tap`. \
             A key still held by an earlier tap is released first, so every tap counts as a new \
             press.",
            input::press_key,
        )
        .method(
            "game/click",
            "{position, button?}",
            "Moves the mouse pointer to `position` in logical window pixels and clicks there, \
             which activates buttons and inventory slots. `button` is `primary` (the default) or \
             `secondary`. game/ui lists where text is drawn.",
            input::click,
        )
}

fn add_setup_methods(methods: GameMethods) -> GameMethods {
    methods
        .method(
            "game/teleport_player",
            "{position}",
            "Moves the player to a world position.",
            setup::teleport_player,
        )
        .method(
            "game/spawn_enemies",
            "{kind, positions, facing?, aggro?}",
            "Spawns enemies of one `EnemyKind` at world positions and returns their entities. \
             `facing` (`up`, `down`, `left` or `right`) turns them until they first move; `aggro` \
             starts them targeting the player as a hit would.",
            setup::spawn_enemies,
        )
        .method(
            "game/despawn_enemies",
            "",
            "Removes every enemy, such as an arena's own before a scripted fight.",
            setup::despawn_enemies,
        )
        .method(
            "game/set_player_health",
            "{hp, max_hp}",
            "Sets the player's health.",
            setup::set_player_health,
        )
        .method(
            "game/set_player_mana",
            "{mana, max_mana?}",
            "Sets the player's mana.",
            setup::set_player_mana,
        )
        .method(
            "game/set_invincible",
            "{invincible}",
            "Makes the player immune to damage, or vulnerable again.",
            setup::set_invincible,
        )
        .method(
            "game/give_item",
            "{item}",
            "Adds a new item to the player's inventory and returns its entity. `item` is one of \
             `sword`, `axe`, `freeze_axe`, `fire_staff`, `ice_staff`, `knight_shield`, \
             `magic_shield`, `health_potion` or `tome_of_healing`.",
            setup::give_item,
        )
        .method(
            "game/use_item",
            "{item, action}",
            "Equips, unequips, consumes or drops one of the player's items, through the same \
             events as the inventory screen, without opening it. `item` is an entity from the \
             snapshot's `player.inventory` and `action` is `equip`, `unequip`, `consume` or \
             `drop`.",
            setup::use_item,
        )
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "remote method handlers return `BrpResult`"
)]
fn list_methods(In(_): In<Option<Value>>, help: Res<GameMethodHelp>) -> BrpResult {
    let methods: Vec<Value> = help
        .iter()
        .map(|method| {
            json!({
                "method": method.name,
                "params": method.params,
                "summary": method.summary,
            })
        })
        .collect();

    Ok(json!(methods))
}

/// Reads a method's params; a call without params reads as `{}`, so all-optional params work.
pub(super) fn parse_params<T: for<'de> Deserialize<'de>>(params: Option<Value>) -> BrpResult<T> {
    parse(params.unwrap_or_else(|| json!({})))
}

/// The error for params that parse but name something the game doesn't have.
pub(super) fn invalid_params(message: impl Into<String>) -> BrpError {
    BrpError {
        code: error_codes::INVALID_PARAMS,
        message: message.into(),
        data: None,
    }
}

/// Reads an entity written by [`entity_id`].
pub(super) fn parse_entity(id: u64) -> BrpResult<Entity> {
    Entity::try_from_bits(id).ok_or_else(|| invalid_params(format!("{id} is not an entity")))
}

/// An entity as every `game/…` method writes and reads it.
pub(super) fn entity_id(entity: Entity) -> u64 {
    entity.to_bits()
}

/// A transform's world position as `[x, y]`.
pub(super) fn position(transform: &Transform) -> [f32; 2] {
    transform.translation.truncate().to_array()
}

/// A value's `Debug` text, which for a fieldless enum is its variant name.
pub(super) fn debug_name(value: impl Debug) -> String {
    format!("{value:?}")
}

use bevy::prelude::*;
use bevy::remote::{BrpResult, RemotePlugin};
use serde_json::{Value, json};

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
    pub(super) fn new() -> Self {
        Self {
            remote_plugin: RemotePlugin::default(),
            help: Vec::new(),
        }
    }

    /// Registers `handler` as the method `name`. `params` lists the method's params the way
    /// a caller writes them, with `?` after the optional ones.
    pub(super) fn method<M>(
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

#[expect(
    clippy::unnecessary_wraps,
    reason = "remote method handlers return `BrpResult`"
)]
pub(super) fn list_methods(In(_): In<Option<Value>>, help: Res<GameMethodHelp>) -> BrpResult {
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

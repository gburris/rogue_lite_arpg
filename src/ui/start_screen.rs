use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::primitives::{gold_button, splash_screen};
use crate::prelude::*;

#[derive(Component, Default, Clone)]
pub struct StartScreen;

pub(super) fn scene() -> impl Scene {
    bsn! {
        @splash_screen(
            AppState::StartScreen,
            "Baba Yaga",
            begin_button(),
            "She is a mysterious witch and ogress from Slavic folklore",
        )
        StartScreen
    }
}

fn begin_button() -> impl Scene {
    bsn! {
        @gold_button("BEGIN", 48.0)
        Node {
            width: px(300),
            height: px(80),
            margin: px(20),
        }
        on(|_: On<Activate>, mut next_app_state: ResMut<NextState<AppState>>| {
            next_app_state.set(AppState::AssetLoading);
        })
    }
}

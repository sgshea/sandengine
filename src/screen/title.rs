//! The title screen that appears when the game starts.

use bevy::prelude::*;

use super::Screen;
use crate::{ui::prelude::*, WorldSizes};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), enter_title);

    app.register_type::<TitleAction>();
    app.add_systems(Update, handle_title_action.run_if(in_state(Screen::Title)));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum TitleAction {
    Play(WorldSizes),
    /// Exit doesn't work well with embedded applications.
    #[cfg(not(target_family = "wasm"))]
    Exit,
}

fn enter_title(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Title))
        .with_children(|children| {
            children.button("Play (Small World)").insert(TitleAction::Play(WorldSizes::Small));
            children.button("Play (Regular World)").insert(TitleAction::Play(WorldSizes::Medium));
            children.button("Play (Huge World)").insert(TitleAction::Play(WorldSizes::Large));

            #[cfg(not(target_family = "wasm"))]
            children.button("Exit").insert(TitleAction::Exit);
        });
}

fn handle_title_action(
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_world_size: ResMut<NextState<WorldSizes>>,
    mut button_query: InteractionQuery<&TitleAction>,
    #[cfg(not(target_family = "wasm"))] mut app_exit: EventWriter<AppExit>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                TitleAction::Play(size) => {
                   next_screen.set(Screen::Playing);
                   next_world_size.set(*size);
                },

                #[cfg(not(target_family = "wasm"))]
                TitleAction::Exit => {
                    app_exit.send(AppExit::Success);
                }
            }
        }
    }
}

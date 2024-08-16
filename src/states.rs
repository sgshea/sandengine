use bevy::prelude::*;

// Controls if debug UI is shown
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum DebugState {
    #[default]
    None,
    ShowAll,
}
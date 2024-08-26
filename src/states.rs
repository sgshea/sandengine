use bevy::prelude::*;

/// Controls if debug UI is shown
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum DebugState {
    #[default]
    None,
    ShowAll,
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

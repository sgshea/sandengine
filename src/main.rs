use bevy::prelude::*;
use sandengine::AppPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

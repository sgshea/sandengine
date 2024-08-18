# sandengine
Falling sand engine written in rust with bevy.

## Features
- Chunked sand simulation in order to use multithreading and 'dirty rectangle' optimization
- Integration with Rapier physics engine for rigid body physics (currently only 1-way interaction with sand simulation)

# Performance
See [performance.md](./performance.md)

## Builds
Release builds are built through actions and can be found on the releases page.
The web version is uploaded onto the GitHub pages at https://sgshea.github.io/sandengine/

Locally, the project can be run with `cargo run` or `cargo run --release`

# License
Apache 2.0: see [LICENSE](./LICENSE)

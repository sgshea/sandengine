# sandengine
Falling sand engine written in rust with bevy.

## Features
High performance sand simulation, using multithreaded and calculated by chunk.

- Rigidbodies by integrating rapier physics engine

# Performance
See [performance.md](./performance.md)

# Bulding
Build instructions contained in the [justfile](./justfile)

## For Web (WASM)
Building requires `wasm-bindgen-cli` and an http server.
- Run `just install-wasm-deps` to get both
- Run `just build-run-wasm` to build and run the wasm build

# License
Apache 2.0: see [LICENSE](./LICENSE)

# Performance
A document to keep track of performance optimizations over time.

All trials are done with 512x512 world size, with 8x8 chunks.
Computer specifications:
- Ryzen 7 7700X (16 Cores)
- Radeon 6800XT

## Method
Times taken using the `Tracy` profiler GUI as described in the [Bevy Profiling Documentation](https://github.com/bevyengine/bevy/blob/main/docs/profiling.md)

Launch Tracy then the game with the following command:
```cargo run --release --features bevy/trace_tracy```
- I then run the simulation for a bit, drawing sand or testing otherwise, then stop the profiler and look at the mean results in Tracy.

## Versions
### v0.1.0
- No multithreading, or dirty chunks
- Basic physics integration only collider generation

- 4x4 Chunks, 64x64 size, 65536 cells

#### update_pixel_simulation
- No sand: 330us
- Lots of sand: 490us
#### Create colliders
- 3.56ms mean after drawing lots of sand but increases above 10ms
### v0.1.1
- Initial dirty chunks implementation
### v0.2.0
- Multithreading, fix dirty chunks
    - Fix for dirty chunks was to check if a chunk was empty or not before updating, this gave most performance gains (to around 35-45us while few updates are happening)
    - Multithreading implementation using [bevy_tasks](https://docs.rs/bevy_tasks/latest/bevy_tasks/index.html), specifically the `ComputeTaskPool`

- No sand: 35us mean
- Drawing lots of sand at one: still around 490us
- Some constant movement in a few chunks such as water/smoke: 270us

Multithreading should help most when the world and chunk size gets much bigger.

Currently multithreading is only implemented in one spot (simulating a chunk), but I want to use the taskpool for other things in the future such as:
- Creating the dirty rectangles of chunks that need updating after simulation while still in thread (will probably need to use a threadsafe hashmap and/or mutexes)
- Updating the render image after simulation while in the thread, currently the whole image is created each frame and is a slow point, but we could just update chunks that have changed
- Creating the colliders for each chunk, this should also not recreate colliders for chunks that haven't changed, collider generation is currently the slowest part of the simulation

> Note on multithreading on web (WASM)
> Currently bevy does not support multithreaded execution on WASM builds, and the `bevy_tasks` module does not support multithreading.
> However, `bevy_tasks` support for web [is merged and due for bevy 0.15 (next release)](https://github.com/bevyengine/bevy/pull/13889)
> The project still works on WASM but this means performance is abit slower than native, optimizations from the dirty chunk system still help a lot
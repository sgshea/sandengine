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
### v0.1.0
- Initial dirty chunks implementation
- Fix dirty chunks
    - Fix for dirty chunks was to check if a chunk was empty or not before updating, this gave most performance gains (to around 35-45us while few updates are happening)

- No sand: 35us mean
- Drawing lots of sand at one: still around 490us
- Some constant movement in a few chunks such as water/smoke: 270us

### v0.1.5
Switch to chunk-based rendering. No use of multithreading here, performance is good enough.
- Down to 1us for the update_chunk_display system when nothing needs to be updated. Seeing around 20-35us when lots are updating.
- This is a good improvement, before was taking around 600us constant *every frame*.
- Stutter on load however, as more images need to be created at start

- Also switched to no scaling, so cells are 1-1 pixels which makes quality better/more consistent

### v0.4.0
Refactor to add-back multithreading for both chunk collider generation and the pixel world updating.
Using bevy_task's ComputeTaskPool and channels.

Chunk collider generation went from around 2-3ms+ to 1ms on regular sized world.
Pixel world updating mostly helped when lots of chunk updating on larger worlds.

> Note on multithreading on web (WASM)
> Currently bevy does not support multithreaded execution on WASM builds, and the `bevy_tasks` module does not support multithreading.
> However, `bevy_tasks` support for web [is merged and due for bevy 0.15 (next release)](https://github.com/bevyengine/bevy/pull/13889)
> The project still works on WASM but this means performance is abit slower than native, optimizations from the dirty chunk system still help a lot
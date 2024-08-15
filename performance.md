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
- Dirty chunks implementation
#### update_pixel_simulation
- No sand: 350us
- Lots of sand: 445us
- After lots of sand drawn (litle movement): back to ~350us


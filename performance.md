# Performance
A document to keep track of performance optimizations over time.

All trials are done with 512x512 world size, with 8x8 chunks.
Computer specifications:
- Ryzen 7 7700X (16 Cores)
- Radeon 6800XT

### Last version using singlethreaded design
Test with 512x512 world size, 8x8 chunks
(render ~ 3.8ms)

No sand: ~ 0.96ms

Lots of sand drawing: ~ (3.4 - 5.0)ms

### Naive multithreading implementation:
Using rayon we are easily able to multithread both the simulation and rendering (image construction from cells).

(render ~ 0.99ms) (284% improvement)

No sand ~ 0.31ms (67.7% improvement)

Lots of sand drawing ~ (0.6 - 1.05)ms (79% improvement)

#### Future Performance
The performance for now is *good enough*, and next task is to integrate rapier2d for rigidbodies.
However there are several major performance gainst that can be had still:
- Track which chunks need to update, if there are only empty cells we could skip both simulation and rendering within a chunk
    - Dirty rect inside chunks, to only update parts with moving cells
- Eventual for larger worlds and moving character (camera) perspectives: caching inactive chunks and/or saving to longer term storage

Possibly migrate to using bevy_task's threadpool if it provides benefits in integration with bevy.
- Currently bevy_task is not as performant is may migrate to use rayon itself [see #11995](https://github.com/bevyengine/bevy/pull/11995)
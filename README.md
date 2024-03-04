# sandengine
Falling sand engine written in rust with bevy.

## Features
Current:
- Single-threaded simulation
    - Chunked (in preparation to multithread)

Planned:
- Multithread
- Dirty rects
- Rigidbodies by integrating rapier physics engine

## Performance
### Last version using singlethreaded design
Test with 512x512 world size, 8x8 chunks
(render ~ 3.8ms)

No sand: ~ 0.96ms

Lots of sand drawing: ~ (3.4 - 5.0)ms

### Naive multithreading implementation:
- Added rayon
- Switched out hashbrown hashmap for std hashmap (to use w/ rayon)

(render ~ 7.1ms): due to hashmap implementation
- Tried using hashbrown w/ rayon which returns render to ~3.8ms but added ~0.15ms to simulation
- Rendering can probably be improved by using rayon on it & rendering by chunk (+ only updating chunks which have changed)

No sand ~ 0.3ms (68.75% improvement)

Lots of sand drawing ~ (0.6 - 1.05)ms (79% improvement)

### Future
Possibly migrate to using bevy_task's threadpool.
- Currently bevy_task is not as performant is may migrate to use rayon itself [see #11995](https://github.com/bevyengine/bevy/pull/11995)
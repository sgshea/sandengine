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
- Switched out bevy hashbrown hashmap for std hashmap (to use w/ rayon)

(render ~ 7.1ms): probably due to hashmap implementation, will need to get one with better hashing algorithm

No sand ~ 0.3ms (68.75% improvement)

Lots of sand drawing ~ (0.6 - 1.05)ms (79% improvement)

### Future
Should try to figure out the preferred solution for multithreading this kind of work in bevy.
- bevy_tasks?

Should try to take advantage of bevy re-export of hashbrown
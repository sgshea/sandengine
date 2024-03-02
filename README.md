# sandengine
Falling sand engine written in rust with bevy.

## Features
Current:
- Single-threaded simulation
    - Chunked (in preparation to multithread)

Planned:
- Multithread
- Rigidbodies by integrating rapier physics engine

## Performance
Test with 512x512 world size, 8x8 chunks
(render ~ 3.8ms)

No sand: ~ 0.96ms

Lots of sand drawing: ~ (3.4 - 5.0)ms
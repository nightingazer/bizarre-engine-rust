<p align="center">
  <img src="https://github.com/nightingazer/bizarre-engine-rust/assets/80068087/1a2685a4-7609-409d-b07c-fb6d39552e32">
</p>

# Bizarre Engine #

My first attempt to make a game engine in Rust. It's will be definitely a very
strange, pretty possibly obnoxious in some aspects, and overall bizarre piece of
software.

The sole purpose of that venture is to conceptualize principals of game engine
building and maybe make something interesting along the way.

I mostly trying to make everything from scratch where it possible, because
reinventing the wheel is fun.

As you can see the project is **highly** in WIP state both from concept and
implementation perspective. Rest assured, we are working on it.

## Rough road map ##

- [x] Logging
- [x] Window creation (winit)
- [ ] Debug assertions and clean shutdowns on critical failures
- [x] Input handling
- [x] Event system (I gave up, so I've used the specs event system)
- [ ] Rendering with Vulkan (in progress)
- [ ] Material system
- [ ] Mutlithreading
- [x] ECS (I've given up, so it specs)
- [ ] Asset management
- [ ] Scene management and serialization
- [ ] Physics
- [ ] Audio
- [ ] Scripting
- [ ] Editor

Somewhere in between there will be some of those:

- [ ] Custom user interface for in-engine and in-game use
- [ ] AI? (obviously the primitive game AI, not the ML stuff, lol)

For forseable future I'm not planning on adding support for DX, Metal or OpenGl,
because Vulkan is pretty much cross-platform and on top of that it's supported
on all the reasonably new GPUs

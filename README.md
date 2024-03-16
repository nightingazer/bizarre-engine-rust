<p align="center">
  <img src="https://github.com/nightingazer/bizarre-engine-rust/assets/80068087/1a2685a4-7609-409d-b07c-fb6d39552e32">
</p>

# Bizarre Engine #

This is a game engine I'm making from scratch in Rust. I don't have a lot of experience in game dev 
and I definitely have even less experience in game engine building. It will definitely be a very
strange, pretty possibly obnoxious in some aspects, and overall bizarre piece of
software.

As I have a lot of third-party packages I use in this engine, I'm still trying to make as much
as I can from scratch. Some of the dependecies will be replaced by my own implementation
as I gain experience and knowledge along the way.

I'm not considering adding DX, Metal or OpenGL support.

Also, I'm going to use this engine to make a game for my university course on game development.

## What have I done to this point ##

* ECS (I've adopted specs for that)
* Event system (also, it's specs)
* Input handling
* Vulkan renderer (it's still work in progress, I'm working on material system at the time)
* Runtime shader compilation
* Multithreaded logging system with configurable loggers
* Mesh loading

## What is yet to be done ##

* Physics system
* Hot reload for game code and assets
* Audio system
* Rebindable controls based on events and input handling

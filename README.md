# unreal-rust

A Rust integration for Unreal Engine

[![Build Status](https://github.com/MaikKlein/ash/workflows/CI/badge.svg)](https://github.com/MaikKlein/ash/actions?workflow=CI)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![LICENSE](https://img.shields.io/badge/license-apache-blue.svg)](LICENSE-APACHE)

## ⚠️Warning⚠️
This project is not ready to be used in a real project. The project is in a very early state and nothing more that a proof of concept right now. The API will change quite frequently. Things might crash, or not work as intended. The user experience will not be great.

I am releasing `unreal-rust` on github to develop it in the open.

## Overview
- Hot reloading (Editor and in play mode) with serialization
- No engine modification, just drop in the `unreal-rust` plugin
- Editor isolation. Unable to crash the edtior in safe code
- Blueprint integration
- Built on top of an entity component system
- A new gameplay framework built on top of `AActor` that provides common functiolity like spawning actors, doing physics sweeps, handling input, logging, animations, rendering and more.
- Does not support the C++ gameplay framework such as `AGameState`, `AGameMode`, `ACharacter` etc.
- Dual licensed under MIT/APACHE

## Supported versions

- 5.0EA2

## Why?

Unreal engine is complete solution to create games ranging from a world class renderer to user friendly tooling. While Unreal comes with C++ support for writing gameplay, it does have some problems.
* Hot reloading can fail
* No engine/editor isolation, which means you can easily crash the whole edtior
* Runs on 1 game thread. Essentially impossible to parallelize.
* Cache unfriendly
* (TODO reword this) Comes with a huge gameplay framework which forces you to design your game the "unreal" way. This is not a bad thing, it is just a different philosophy. Unity for example does not impose high level concept like pawns, gamestate, gamemodes on you, but provides high level apis for creating a game. Such as getting input, doing physics etc.

`unreal-rust` solves all of the issues above. The main intention is that it should make unreal feel much more like framework where you can do whatever you want. You get fast iteration times with hot reloading based on dll reloading.

You can not crash the editor in safe code. If you mess up and `panic` in your Rust code, `unreal-rust` will catch the `panic` and simply exit playmode.

`unreal-rust` is built on top of an entity component system with benefits such as automatic parallelism, and fast memory access patterns.

## Experiences

While `unreal-rust` is in its infancy I did prototype a very simple kinematic character controller with it. The experience was absolutely fantastic. I could iterate with with <1second build times and live hot reloading.
As this is the first character controller that I have written, I did a lot of beginner mistakes as there are quite a few edge cases that you need to handle.
Because the iteration times were so fast, I could just try out new approaches very quickly. I never managed to crash my editor.
I did use unreal's visual logger quite frequently, which allows to you record  

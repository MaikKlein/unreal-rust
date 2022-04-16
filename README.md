# unreal-rust

A Rust integration for Unreal Engine

[![Build Status](https://github.com/MaikKlein/unreal-rust/workflows/CI/badge.svg)](https://github.com/MaikKlein/unreal-rust/actions?workflow=CI)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![LICENSE](https://img.shields.io/badge/license-apache-blue.svg)](LICENSE-APACHE)

## ⚠️Warning⚠️

This project is not ready to be used in a real project. The project is in a very early state and nothing more that a proof of concept right now. The API will change quite frequently. Things might crash, or not work as intended. The user experience will not be great.

I am releasing `unreal-rust` on github to develop it in the open.

## Features

- Simple opinionated bindings: Easy access to APIs like playing sounds, spawning actors, doing physics while leaving almost the whole gameplay framework behind such as `Pawns`, `Characters`, `GameMode`, `GameState` etc.
- Developer friendly: Fast iteration times with hot reloading in the editor and during live play both on Windows and Linux. Unable to crash the editor in safe code, all panics are caught and will just throw you out of play mode.
- Fast: Built on top of an entity component system.
- Practical: You still have access to blueprints. You don't need to leave behind your animations blueprints just because you are using `unreal-rust`. Just access the data you want like `velocity` to drive your animations.
- No magic: Built on top of simple C FFI. This allows Unreal to call into Rust, and Rust to call into Unreal.
- Easy to get started: Simply drop the `RustPlugin` into your projects `Plugin` folder and you are ready to go. No engine modifications necessary.
- Free: Dual licensed under MIT/APACHE

## Known problems

- No console support. I simply can not offer console support as it is a very closed off ecosystem. Nor do I have access to any developer kits myself.
- This is just a hobby project of mine that I work on outside of my normal work hours. I might be slow to respond to issues, questions, feature requests, or PR reviews.


## Getting started

### Running the example

- Clone this repository `git clone https://github.com/MaikKlein/unreal-rust`
- Make sure to clone the submodules as well `git submodule update --init`
- Run the setup
- - Linux `sh setup.sh`
- - Windows Copy or symlink `RustPlugin` into `examples/RustExample/Plugins`
- TODO Rust
- Build `example/RustExample`
- - I recommend installing [ue4cli](https://docs.adamrehn.com/ue4cli/overview/introduction-to-ue4cli)
- - `cd example/RustExample`
- - `ue4 build Development Editor`
- - and run the editor `ue4 run`

## Supported versions

- `5.0`

This project will always try to support the latest version.

- Latest version of Unreal
- Latest version of Rust
- Latest version of all dependencies

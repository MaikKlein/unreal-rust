# 🦀 unreal-rust

Opinionated Rust integration for Unreal Engine

[![Build Status](https://github.com/MaikKlein/unreal-rust/workflows/CI/badge.svg)](https://github.com/MaikKlein/unreal-rust/actions?workflow=CI)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![LICENSE](https://img.shields.io/badge/license-apache-blue.svg)](LICENSE-APACHE)

## ☣️ Warning

This project is not ready to be used in a real project. The project is in a very early state and nothing more that a proof of concept right now. The API will change quite frequently. Things might crash, or not work as intended. The user experience will not be great.

I am releasing `unreal-rust` on github to develop it in the open.

## 🎯 Features

- **Simple opinionated bindings**: Easy access to core APIs like playing sounds, spawning actors, pathfinding, physics etc. 
- **Developer friendly**: Fast iteration times with hot reloading in the editor and during live play. Panics are caught and do not crash the editor
- **Editor integration**: Add Rust components to actors in the editor, or access Rust components from Blueprint to drive animations. 
- **Entity Component System**: unreal-rust is built on top of an ECS.
- **Built on top of `AActor`**: Most gameplay features like `GameMode`, `Characters`, `GameState`, `GAS` are not directly accessible in unreal-rust. Instead unreal-rust will provide optional alternatives. But you can still interact with most parts of the engine as Rust components can be accessed in Blueprint.
- **No engine modifications**: unreal-rust is only a `Plugin`, just drop it in your project. See [Supported versions](#supported-versions) for more information.
- **Samples**: The development of unreal-rust is heavily driven by samples.
- **Free**: Dual licensed under MIT/APACHE

## 🚩 Known problems

- No console support. I simply can not offer console support as it is a very closed off ecosystem. Nor do I have access to any developer kits myself.
- This is just a hobby project of mine that I work on outside of my normal work hours. I might be slow to respond to issues, questions, feature requests, or PR reviews.


## 🦮 Getting started

### Running the example

_I am aware that these are a lot of steps. I am sorry, I will try to simplify this in the future_

- Clone this repository `git clone https://github.com/MaikKlein/unreal-rust`
- Make sure to clone the submodules as well `git submodule update --init`

- Run the setup
- - Linux `sh setup.sh`
- - Windows Copy or symlink `RustPlugin` into `examples/RustExample/Plugins`
- Build the rust code
- - `cargo build --release`
- - Copy the dll/so file into the project `cp target/release/libunreal_rust_example.so example/RustExample/Binaries/rustplugin.so`
- - Bonus: You can use `cargo-watch` to automatically rebuild your code on file changes. `cargo watch -i "*.so" -s "cargo build --release; cp target/release/libunreal_rust_example.so example/RustExample/Binaries/rustplugin.so"` You can install it with `cargo install cargo-watch`

- Build `example/RustExample`
- - I recommend installing [ue4cli](https://docs.adamrehn.com/ue4cli/overview/introduction-to-ue4cli)
- - `cd example/RustExample`
- - `ue4 build Development Editor`
- - and run the editor `ue4 run`

### Minimal

- Move the `RustPlugin` folder into your projects `Plugins` folder
- Start with the `example/minimal`
- Run `cargo build --release`. This will produce a C dll in `target/release`
- Copy the dll into the `Binaries` folder of your project, and name it `rustplugin.so`

## Supported versions

- `5.0`

This project will always try to support the latest version.

- Latest version of Unreal
- Latest version of Rust
- Latest version of all dependencies

## FAQ

### How does the C FFI work?

We define the C FFI inside Rust like `fn set_actor_position(pos: Vec3)`. We generate a C header with bindgen. We then implement `set_actor_position` in C++. When we load the Rust dll from within Unreal, we pass in the function pointers like `set_actor_position` into the Rust dll. This allows Rust to call functions of Unreal.

Additonally we also define and implement C functions in Rust. We then pass the function pointers of those functions into Unreal when the dll is loaded. This allows Unreal to call into Rust. For example Unreal calls `Tick` every frame which is define inside the Rust dll.

## Alternatives

* [Unreal Angelscript](https://angelscript.hazelight.se/)
* [UnrealCLR](https://github.com/nxrighthere/UnrealCLR)

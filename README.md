# 🦀 unreal-rust

A Rust integration for Unreal Engine

[![Build Status](https://github.com/MaikKlein/unreal-rust/workflows/CI/badge.svg)](https://github.com/MaikKlein/unreal-rust/actions?workflow=CI)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![LICENSE](https://img.shields.io/badge/license-apache-blue.svg)](LICENSE-APACHE)

## ☣️ Warning

This project is not ready to be used in a real project. The project is in a very early state and nothing more that a proof of concept right now. The API will change quite frequently. Things might crash, or not work as intended. The user experience will not be great.

I am releasing `unreal-rust` on github to develop it in the open.

## 🎯 Features

- **Simple opinionated bindings**: Easy access to APIs like playing sounds, spawning actors, doing physics while leaving almost the whole gameplay framework behind such as `Pawns`, `Characters`, `GameMode`, `GameState` etc. The API will be closer to Unity.
- **Developer friendly**: Fast iteration times with hot reloading in the editor and during live play both on Windows and Linux. Unable to crash the editor in safe code, all panics are caught and will just throw you out of play mode.
- **Fast**: Built on top of an entity component system.
- **Practical**: `unreal-rust` is not an all or nothing solution. You can still use blueprints if you want to and drive your animation blueprints by reading state out of `unreal-rust`. The example accesses 
- **No magic**: Built on top of simple C FFI. This allows Unreal to call into Rust, and Rust to call into Unreal.
- **Easy to get started**: Simply drop the `RustPlugin` into your projects `Plugin` folder and you are ready to go. No engine modifications necessary.
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

### Why?

Unreal is a fantastic engine that comes with a huge amount of features that are ready to be used in a real game. I just have not been a huge fan of the gameplay framework. Compiling the C++ gamecode is quite slow, there is a lot of magic behind macros, hot reloading was not reliable, and you can crash the editor in gameplay code.

I much prefer Unity. It is quite easy to get started and its very unopinionated. Sadly from an outsiders perspective it also seems to be in a weird state right now with DOTS. You also don't access to the source code and the whole engine is hidden away from you, unless you buy a license.

I wanted to see if I can make Unreal more like Unity and fix all the issues I have with it. This is how `unreal-rust` was born. I want hotreloading to always work even on Linux. `unreal-rust` simply reloads a dll. I never want to accidentally crash the editor. `unreal-rust` will catch all panics and exit playmode.
I also want live reloading so I can tweak code live while play testing it. Some gameplay code like character controllers are quite difficult to write and having a good iteration workflow is very important to have. You need to be able to test out ideas in a very short amount of time. I can make changes to the gameplay code in <1s and have it hot reloaded in the editor.

There also already usable alternatives around like [Unreal Angelscript](https://angelscript.hazelight.se/) that fix most of my issues with Unreal.

// TODO







_I am aware that you can write your own engine, but not everyone has the expertise to do that and unless it is a 2d game or you heavily downscope your project its just not going to happen._

### How does the C FFI work?

We define the C FFI inside Rust like `fn set_actor_position(pos: Vec3)`. We generate a C header with bindgen. We then implement `set_actor_position` in C++. When we load the Rust dll from within Unreal, we pass in the function pointers like `set_actor_position` into the Rust dll. This allows Rust to call functions of Unreal.

Additonally we also define and implement C functions in Rust. We then pass the function pointers of those functions into Unreal when the dll is loaded. This allows Unreal to call into Rust. For example Unreal calls `Tick` every frame which is define inside the Rust dll.

### Why Rust?

// TODO improve

Splitting gameplay code into a separate dll to improve iteration times could have been done in any language but there are a few reasons why I chose Rust

Rust is an very developer friendly language. It gives you best in class error messages. It has a more strict compiler that catches a lot of errors at compiletime. Easy to use build system. `cargo build` will just work with most project. No need to read instructions.
No segfaults in normal code, crashes or panics are well defined.
Absolute fantastic documentation engine, just run `cargo doc`. Have a look at the [Standard library](https://doc.rust-lang.org/stable/std/)
No frills package manager that makes it easy to add external libraries. And because Rust is a safe language, no need to worry that the dependencies you bring in will do bad things (most of the time).


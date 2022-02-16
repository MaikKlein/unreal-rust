# unreal-rust

A Rust integration for Unreal Engine

[![Build Status](https://github.com/MaikKlein/unreal-rust/workflows/CI/badge.svg)](https://github.com/MaikKlein/unreal-rust/actions?workflow=CI)
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
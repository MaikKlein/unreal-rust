use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        // TODO: This is stupid and easy to forgot to add things here. Can we auto detect those?
        .include_item("UnrealBindings")
        .include_item("RustBindings")
        .include_item("CreateRustBindings")
        .include_item("EntryUnrealBindingsFn")
        .include_item("EntryBeginPlayFn")
        .include_item("EntryTickFn")
        .include_item("RetrieveUuids")
        .include_item("ActorSpawnedEvent")
        .include_item("ActorBeginOverlap")
        .include_item("ActorEndOverlap")
        .include_item("ActorHitEvent")
        .include_item("ActorDestroyEvent")
        .with_pragma_once(true)
        //.with_config(Config {
        //    structure: StructConfig  {
        //        derive_constructor: true,
        //        ..Default::default()
        //    },
        //    ..Default::default()
        //})
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("../RustPlugin/Source/RustPlugin/Public/Bindings.h");
}
